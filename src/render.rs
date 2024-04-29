
use bytemuck::{Pod, Zeroable};
use wgpu::RenderPipeline;
use std::borrow::Cow;
use wgpu::util::DeviceExt;
use wgpu_text::{glyph_brush::{ab_glyph::FontRef, Section as TextSection, Text}, BrushBuilder, TextBrush};
use winit::window::Window;

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

const INDICES: &[u16] = &[
    0, 1, 3,    // A, B, D
    3, 1, 2,    // D, B, C
];

pub struct Render<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,

    verticies: Box<[Vertex]>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    text_brush: TextBrush<FontRef<'a>>,
    pub text: String,
}

impl<'a> Render<'a> {
    pub async fn new(window: &'a Window) -> Render<'a> {
        let mut size = window.inner_size();
        size.width = size.width.max(500);
        size.height = size.height.max(500);
        let (surface, adapter, device, queue) = init_screen(&window).await;
        let mut config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);

        let render_pipeline = setup_render(&device, &surface, &adapter, &config);
        let (vertex_buffer, index_buffer, verticies) = init_vertices(&device);
        let text_brush = init_text(&device, &config);
        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            verticies,
            vertex_buffer,
            index_buffer,
            text_brush,
            text: "START".to_string(),
        }
    }

    pub fn render(&mut self) -> () {
        let frame = self.surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: None,
            });
        {
            let mut rpass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(
                                wgpu::Color{ // GREY
                                    r: 0.66, // 167/256
                                    g: 0.66,
                                    b: 0.66,
                                    a: 1.0,
                                }),
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
            rpass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);

            let text_section = TextSection::default().add_text(Text::new(&self.text));
            self.text_brush.queue(&self.device, &self.queue, vec![&text_section]).unwrap();
            self.text_brush.draw(&mut rpass);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn move_square(&mut self, x: f32, y: f32) -> () {
        let size: f32 = 0.25;
        self.verticies = Box::new([
            Vertex { position: [x, y-size], color: [1.0, 0.0, 0.0] },     // A
            Vertex { position: [x + size, y-size], color: [0.0, 1.0, 0.0] },    // B
            Vertex { position: [x + size, y], color: [0.0, 0.0, 1.0] },     // C
            Vertex { position: [x, y], color: [1.0, 1.0, 0.0] },      // D);
        ]);

        self.vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&*self.verticies),
            usage: wgpu::BufferUsages::VERTEX,
        });
    }
}

fn init_text<'a>(device: & wgpu::Device, config: &wgpu::SurfaceConfiguration) -> TextBrush<FontRef<'a>> {
    let font: &[u8] = include_bytes!("../fonts/DejaVuSans.ttf");
    let mut text_brush = BrushBuilder::using_font_bytes(font).unwrap()
     /* .initial_cache_size((16_384, 16_384))) */ // use this to avoid resizing cache texture
        .build(&device, config.width, config.height, config.format);
    //let mut label_text = "START".to_string();
    return text_brush;
}

fn init_vertices(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer, Box<[Vertex]>) {
    let verticies = Box::new([
        Vertex { position: [-0.5, -0.5], color: [1.0, 0.0, 0.0] },     // A
        Vertex { position: [0.5, -0.5], color: [0.0, 1.0, 0.0] },    // B
        Vertex { position: [0.5, 0.5], color: [0.0, 0.0, 1.0] },     // C
        Vertex { position: [-0.5, 0.5], color: [1.0, 1.0, 0.0] },      // D);
    ]);
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&*verticies),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        }
    );
    return (vertex_buffer, index_buffer, verticies)
}

async fn init_screen<'a>(window: &'a Window) ->
(wgpu::Surface<'a>, wgpu::Adapter, wgpu::Device, wgpu::Queue) {
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
                label: None,
                required_features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    return (surface, adapter, device, queue);
}

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