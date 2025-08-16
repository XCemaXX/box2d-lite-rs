mod draw_primitives;
mod math;

use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;
use wgpu::util::DeviceExt;
use wgpu::{RenderPipeline, Surface};
use wgpu_text::{
    glyph_brush::{ab_glyph::FontRef, Section as TextSection, Text},
    BrushBuilder, TextBrush,
};

use crate::buttons::BUTTONS;
use draw_primitives::{create_bordered_rectangle, create_line, create_point};
use physics::primitives::{Line, Point, Rectangle};

use self::draw_primitives::create_triangle;

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

const GRAY_BACKGROUND: wgpu::Color = wgpu::Color {
    r: 0.66, // 167/256
    g: 0.66,
    b: 0.66,
    a: 1.0,
};

const GRAY_BUTTON: wgpu::Color = wgpu::Color {
    r: 0.77,
    g: 0.77,
    b: 0.77,
    a: 1.0,
};

pub struct Size {
    pub width: u32,
    pub height: u32,
}

pub struct Render<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: Size,
    render_pipeline: wgpu::RenderPipeline,

    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    text_brush: TextBrush<FontRef<'a>>,
    pub text: String,
}

impl<'a> Render<'a> {
    pub async fn new(
        window: impl Into<wgpu::SurfaceTarget<'a>>,
        width: u32,
        height: u32,
    ) -> Render<'a> {
        let mut size = Size { width, height };
        size.width = size.width.max(600);
        size.height = size.height.max(600);
        let (surface, adapter, device, queue) = init_screen(window).await;

        let config = config_surface(&surface, &adapter, &size);
        surface.configure(&device, &config);

        let render_pipeline = setup_render(&device, &surface, &adapter, &config);
        let (vertex_buffer, index_buffer, vertices, indices) = init_vertices(&device);
        let text_brush = init_text(&device, &config);
        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
            text_brush,
            text: "START".to_string(),
        }
    }

    pub fn render(&mut self) -> () {
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
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
                    load: wgpu::LoadOp::Clear(GRAY_BACKGROUND),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        rpass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);

        let text_section = TextSection::default().add_text(Text::new(&self.text));
        self.text_brush
            .queue(&self.device, &self.queue, vec![&text_section])
            .unwrap();
        self.text_brush.draw(&mut rpass);
        drop(rpass);

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }

    pub fn update_frame(
        &mut self,
        rectangles: Vec<Rectangle>,
        points: Vec<Point>,
        lines: Vec<Line>,
    ) {
        let mut index = 0_u16;
        self.vertices.clear();
        self.indices.clear();
        (self.vertices, self.indices) = create_buttons(&mut index);
        for r in rectangles {
            let (vertices, indices) = create_bordered_rectangle(r, &mut index, &GRAY_BACKGROUND);
            self.vertices.extend(vertices.iter());
            self.indices.extend(indices.iter());
        }
        for p in points {
            let (vertices, indices) = create_point(&p, &mut index);
            self.vertices.extend(vertices.iter());
            self.indices.extend(indices.iter());
        }
        for l in lines {
            let (vertices, indices) = create_line(&l, &mut index);
            self.vertices.extend(vertices.iter());
            self.indices.extend(indices.iter());
        }
        self.update_buffers();
    }

    fn update_buffers(&mut self) {
        self.vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        self.index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsages::INDEX,
            });
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.size.width = width.max(600);
        self.size.height = height.max(600);
        self.config.width = self.size.width;
        self.config.height = self.size.height;
        // doesn't work. infinite resize
        //self.surface.configure(&self.device, &self.config);
        //self.text_brush.resize_view(self.size.width as f32, self.size.height as f32, &self.queue);
    }
}

fn create_buttons(index_start: &mut u16) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();
    for (_name, b) in BUTTONS {
        let (v, i) = create_bordered_rectangle(b.rect.clone(), index_start, &GRAY_BUTTON);
        vertices.extend(v.iter());
        indices.extend(i.iter());
        let (v, i) = create_triangle(&b.icon, index_start);
        vertices.extend(v.iter());
        indices.extend(i.iter());
    }
    (vertices, indices)
}

fn init_text<'a>(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> TextBrush<FontRef<'a>> {
    let font: &[u8] = include_bytes!("../../../fonts/DejaVuSans.ttf");
    return BrushBuilder::using_font_bytes(font)
        .unwrap()
        /* .initial_cache_size((16_384, 16_384))) */ // use this to avoid resizing cache texture
        .build(&device, config.width, config.height, config.format);
}

fn init_vertices(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer, Vec<Vertex>, Vec<u16>) {
    let (v, i) = (Vec::new(), Vec::new());
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&v),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&i),
        usage: wgpu::BufferUsages::INDEX,
    });
    return (vertex_buffer, index_buffer, v, i);
}

async fn init_screen<'a>(
    window: impl Into<wgpu::SurfaceTarget<'a>>,
) -> (wgpu::Surface<'a>, wgpu::Adapter, wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::default();

    //window.set_cursor_icon(CursorIcon::Grab);
    let surface = instance.create_surface(window).unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("WGPU Device"),
                required_features: wgpu::Features::default(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        )
        .await
        .expect("Failed to create device");
    return (surface, adapter, device, queue);
}

pub fn setup_render(
    device: &wgpu::Device,
    _surface: &wgpu::Surface<'_>,
    _adapter: &wgpu::Adapter,
    config: &wgpu::SurfaceConfiguration,
) -> RenderPipeline {
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

    //let swapchain_capabilities = surface.get_capabilities(&adapter);

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::desc()],
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
        multiview: None,
        cache: None,
    });
    return render_pipeline;
}

fn config_surface(
    surface: &Surface,
    adapter: &wgpu::Adapter,
    size: &Size,
) -> wgpu::SurfaceConfiguration {
    let surface_capabilities = surface.get_capabilities(&adapter);
    let surface_format = surface_capabilities
        .formats
        .iter()
        .copied()
        .find(|f| !f.is_srgb()) // egui wants a non-srgb surface texture
        .unwrap_or(surface_capabilities.formats[0]);

    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_capabilities.present_modes[0],
        alpha_mode: surface_capabilities.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    }
}
