mod gpu;
mod gpu_buffer;
mod mesh;
mod text;

use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;
use wgpu::RenderPipeline;

use crate::letterbox::Rect;
use gpu::Gpu;
use gpu_buffer::GpuBuffer;
pub use mesh::Mesh;
use text::Text;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3];
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

// Text size and its gap from the field edge, in CSS pixels.
const TEXT_SIZE: f32 = 16.0;
const TEXT_PAD: f32 = 4.0;

/// Draws the frame: the mesh in the field, the text on top.
pub struct Render {
    gpu: Gpu,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: GpuBuffer,
    index_buffer: GpuBuffer,
    text: Text,
}

impl Render {
    pub async fn new(
        window: impl Into<wgpu::SurfaceTarget<'static>>,
        width: u32,
        height: u32,
    ) -> Result<Render, String> {
        let gpu = Gpu::new(window, width, height).await?;
        let device = &gpu.device;
        Ok(Self {
            pipeline: setup_render(device, gpu.format),
            vertex_buffer: GpuBuffer::new(device, "Vertex Buffer", wgpu::BufferUsages::VERTEX),
            index_buffer: GpuBuffer::new(device, "Index Buffer", wgpu::BufferUsages::INDEX),
            text: Text::new(device, gpu.format, gpu.size())?,
            gpu,
        })
    }

    /// `mesh` is in clip space of the `field` viewport; `field` is in surface pixels.
    /// `text` starts at the top-left corner of `field`, wrapped and clipped to it.
    /// `surface_per_css`: surface pixels per CSS pixel, to keep text size in CSS pixels.
    pub fn draw(
        &mut self,
        mesh: &Mesh,
        clear: [f32; 3],
        field: Rect,
        surface_per_css: f32,
        text: &str,
    ) {
        let Some(frame) = self.gpu.frame() else {
            return;
        };
        let (device, queue) = (&self.gpu.device, &self.gpu.queue);
        self.vertex_buffer
            .upload(device, queue, bytemuck::cast_slice(&mesh.vertices));
        self.index_buffer
            .upload(device, queue, bytemuck::cast_slice(&mesh.indices));

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: clear[0].into(),
                        g: clear[1].into(),
                        b: clear[2].into(),
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        let (w, h) = self.gpu.size();
        let (w, h) = (w as f32, h as f32);
        // Float rounding must not push the viewport out of the surface.
        let x = field.x.clamp(0.0, w);
        let y = field.y.clamp(0.0, h);
        rpass.set_viewport(x, y, field.w.min(w - x), field.h.min(h - y), 0.0, 1.0);
        if !mesh.indices.is_empty() {
            rpass.set_pipeline(&self.pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice());
            rpass.set_index_buffer(self.index_buffer.slice(), wgpu::IndexFormat::Uint32);
            rpass.draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
        }

        // The text brush maps surface pixels over the whole surface.
        rpass.set_viewport(0.0, 0.0, w, h, 0.0, 1.0);
        let k = surface_per_css;
        let area = field.inset(TEXT_PAD * k);
        self.text
            .draw(device, queue, &mut rpass, area, text, TEXT_SIZE * k);
        drop(rpass);

        queue.submit(std::iter::once(encoder.finish()));
        queue.present(frame);
    }

    /// Returns the size actually used: the GPU limits the surface size.
    pub fn resize(&mut self, width: u32, height: u32) -> (u32, u32) {
        let size = self.gpu.resize(width, height);
        self.text.resize(&self.gpu.queue, size);
        size
    }
}

fn setup_render(device: &wgpu::Device, format: wgpu::TextureFormat) -> RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader_square.wgsl"))),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Some(Vertex::desc())],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}
