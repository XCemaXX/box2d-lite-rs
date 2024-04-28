
use bytemuck::{Pod, Zeroable};
use wgpu::RenderPipeline;
use std::borrow::Cow;

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

pub const VERTICES: &[Vertex] = &[
    Vertex {
        // vertex a
        position: [-0.5, -0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        // vertex b
        position: [0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        // vertex d
        position: [-0.5, 0.5],
        color: [1.0, 1.0, 0.0],
    },
    Vertex {
        // vertex d
        position: [-0.5, 0.5],
        color: [1.0, 1.0, 0.0],
    },
    Vertex {
        // vertex b
        position: [0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        // vertex c
        position: [0.5, 0.5],
        color: [0.0, 0.0, 1.0],
    },
];

pub fn setup_render(device: &wgpu::Device, surface: &wgpu::Surface<'_>,
    adapter: &wgpu::Adapter, config: &wgpu::SurfaceConfiguration) -> RenderPipeline {
    // Load the shaders from disk
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader_square.wgsl"))),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = surface.get_capabilities(&adapter);

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });
    return render_pipeline;
}