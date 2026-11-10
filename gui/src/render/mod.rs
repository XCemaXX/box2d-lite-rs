mod gpu_buffer;
mod mesh;

use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;
use wgpu::RenderPipeline;
use wgpu_text::{
    BrushBuilder, TextBrush,
    glyph_brush::{Section as TextSection, Text, ab_glyph::FontRef},
};

use crate::letterbox::Rect;
use gpu_buffer::GpuBuffer;
pub use mesh::Mesh;

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

// Text size in CSS pixels.
const TEXT_SIZE: f32 = 16.0;

pub struct Render {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: GpuBuffer,
    index_buffer: GpuBuffer,

    text_brush: TextBrush<FontRef<'static>>,
}

impl Render {
    pub async fn new(
        window: impl Into<wgpu::SurfaceTarget<'static>>,
        width: u32,
        height: u32,
    ) -> Result<Render, String> {
        let (surface, adapter, device, queue) = init_screen(window).await?;

        let (width, height) = surface_size(&device, width, height);
        let config = config_surface(&surface, &adapter, width, height);
        surface.configure(&device, &config);

        let render_pipeline = setup_render(&device, &config);
        let vertex_buffer = GpuBuffer::new(&device, "Vertex Buffer", wgpu::BufferUsages::VERTEX);
        let index_buffer = GpuBuffer::new(&device, "Index Buffer", wgpu::BufferUsages::INDEX);
        let text_brush = init_text(&device, &config)?;
        Ok(Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            text_brush,
        })
    }

    /// `mesh` is in clip space of the `field` viewport; `field` is in surface pixels.
    /// `surface_per_css`: surface pixels per CSS pixel, to keep text size in CSS pixels.
    pub fn draw(
        &mut self,
        mesh: &Mesh,
        clear: [f32; 3],
        field: Rect,
        surface_per_css: f32,
        text: &str,
    ) {
        let (device, queue) = (&self.device, &self.queue);
        self.vertex_buffer
            .upload(device, queue, bytemuck::cast_slice(&mesh.vertices));
        self.index_buffer
            .upload(device, queue, bytemuck::cast_slice(&mesh.indices));

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            other => {
                log::warn!("Skip frame: {other:?}");
                return;
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
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

        let (w, h) = (self.config.width as f32, self.config.height as f32);
        // Float rounding must not push the viewport out of the surface.
        let x = field.x.clamp(0.0, w);
        let y = field.y.clamp(0.0, h);
        rpass.set_viewport(x, y, field.w.min(w - x), field.h.min(h - y), 0.0, 1.0);
        if !mesh.indices.is_empty() {
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice());
            rpass.set_index_buffer(self.index_buffer.slice(), wgpu::IndexFormat::Uint32);
            rpass.draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
        }

        // Text is an overlay over the whole surface, in pixels.
        rpass.set_viewport(0.0, 0.0, w, h, 0.0, 1.0);
        let text_section = TextSection::default()
            .add_text(Text::new(text).with_scale(TEXT_SIZE * surface_per_css));
        match self
            .text_brush
            .queue(&self.device, &self.queue, vec![&text_section])
        {
            Ok(()) => self.text_brush.draw(&mut rpass),
            Err(e) => log::warn!("Skip text: {e}"),
        }
        drop(rpass);

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(frame);
    }

    /// Returns the size actually used: the GPU limits the surface size.
    pub fn resize(&mut self, width: u32, height: u32) -> (u32, u32) {
        (self.config.width, self.config.height) = surface_size(&self.device, width, height);
        self.surface.configure(&self.device, &self.config);
        let (w, h) = (self.config.width, self.config.height);
        self.text_brush.resize_view(w as f32, h as f32, &self.queue);
        (w, h)
    }
}

fn init_text(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> Result<TextBrush<FontRef<'static>>, String> {
    let font: &'static [u8] = include_bytes!("../../../fonts/DejaVuSans.ttf");
    let brush = BrushBuilder::using_font_bytes(font)
        .map_err(|e| format!("font: {e}"))?
        .build(device, config.width, config.height, config.format);
    Ok(brush)
}

async fn init_screen(
    window: impl Into<wgpu::SurfaceTarget<'static>>,
) -> Result<
    (
        wgpu::Surface<'static>,
        wgpu::Adapter,
        wgpu::Device,
        wgpu::Queue,
    ),
    String,
> {
    // navigator.gpu can exist without a usable adapter: then fall back to WebGL2.
    let instance = wgpu::util::new_instance_with_webgpu_detection(
        wgpu::InstanceDescriptor::new_without_display_handle(),
    )
    .await;

    let surface = instance
        .create_surface(window)
        .map_err(|e| format!("create surface: {e}"))?;
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
            apply_limit_buckets: false,
        })
        .await
        .map_err(|e| format!("request adapter: {e}"))?;
    log::info!("Backend: {:?}", adapter.get_info().backend);

    // WebGL2 has no compute; WebGPU does. Ask for what the adapter can give.
    let base_limits = if adapter
        .get_downlevel_capabilities()
        .flags
        .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
    {
        wgpu::Limits::downlevel_defaults()
    } else {
        wgpu::Limits::downlevel_webgl2_defaults()
    };

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("WGPU Device"),
            required_features: wgpu::Features::default(),
            // The surface may be as large as the adapter allows.
            required_limits: base_limits.using_resolution(adapter.limits()),
            experimental_features: wgpu::ExperimentalFeatures::default(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
        })
        .await
        .map_err(|e| format!("request device: {e}"))?;
    Ok((surface, adapter, device, queue))
}

fn setup_render(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> RenderPipeline {
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
                format: config.format,
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

/// Zero is not a valid surface size, and the GPU limits the maximum.
fn surface_size(device: &wgpu::Device, width: u32, height: u32) -> (u32, u32) {
    let max = device.limits().max_texture_dimension_2d;
    (width.clamp(1, max), height.clamp(1, max))
}

fn config_surface(
    surface: &wgpu::Surface,
    adapter: &wgpu::Adapter,
    width: u32,
    height: u32,
) -> wgpu::SurfaceConfiguration {
    let surface_capabilities = surface.get_capabilities(adapter);
    let surface_format = surface_capabilities
        .formats
        .iter()
        .copied()
        // Colors are sRGB values written as is: no conversion on store.
        .find(|f| !f.is_srgb())
        .unwrap_or(surface_capabilities.formats[0]);

    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        color_space: wgpu::SurfaceColorSpace::Auto,
        width,
        height,
        present_mode: surface_capabilities.present_modes[0],
        alpha_mode: surface_capabilities.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    }
}
