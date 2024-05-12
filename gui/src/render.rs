use bytemuck::{Pod, Zeroable};
use wgpu::RenderPipeline;
use std::borrow::Cow;
use wgpu::util::DeviceExt;
use wgpu_text::{glyph_brush::{ab_glyph::FontRef, Section as TextSection, Text}, BrushBuilder, TextBrush};
use winit::window::Window;

use crate::primitives::{Rectangle, Point, Line};

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

const GREY_COLOR: wgpu::Color = wgpu::Color {
    r: 0.66, // 167/256
    g: 0.66,
    b: 0.66,
    a: 1.0,
};

const RED_COLOR: [f32; 3] = [1.0, 0.0, 0.0];
const BLACK_COLOR: [f32; 3] = [0.0, 0.0, 0.0];

fn color_as_array(color: wgpu::Color) -> [f32; 3] {
    [color.r as f32, color.g as f32, color.b as f32]
}

pub struct Render<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,

    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    text_brush: TextBrush<FontRef<'a>>,
    pub text: String,
}

impl<'a> Render<'a> {
    pub async fn new(window: &'a Window) -> Render<'a> {
        let mut size = window.inner_size();
        size.width = size.width.max(600);
        size.height = size.height.max(600);
        let (surface, adapter, device, queue) = init_screen(&window).await;
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
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
                            load: wgpu::LoadOp::Clear(GREY_COLOR),
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
            self.text_brush.queue(&self.device, &self.queue, vec![&text_section]).unwrap();
            self.text_brush.draw(&mut rpass);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn update_frame(&mut self, rectangles: Vec<Rectangle>, points: Vec<Point>, lines: Vec<Line>) {
        let mut index = 0_u16;
        self.vertices.clear();
        self.indices.clear();
        for r in rectangles {
            let (vertices, indices) = create_bordered_rectangle(r, &mut index);
            self.vertices.extend(vertices.iter());
            self.indices.extend(indices.iter());
        }
        for p in points {
            let (vertices, indices) = create_point(p, &mut index);
            self.vertices.extend(vertices.iter());
            self.indices.extend(indices.iter());
        }
        for l in lines {
            let (vertices, indices) = create_line(l, &mut index);
            self.vertices.extend(vertices.iter());
            self.indices.extend(indices.iter());
        }
        self.update_buffers();
    }

    fn update_buffers(&mut self) {
        self.vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        self.index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
    }
}

fn create_bordered_rectangle(r: Rectangle, index_start: &mut u16) -> (Vec<Vertex>, Vec<u16>) {
    let corners = math::get_corners(&r);
    const BORDER_WIDTH: f32 = 0.02;
    let mut r = r;
    r.width -= BORDER_WIDTH;
    r.height -= BORDER_WIDTH;
    let inner_corners = math::get_corners(&r);

    let vertices: Vec<Vertex> = vec![
        //outer square
        Vertex { position: corners[0], color: [1.0, 0.0, 0.0] },     // A
        Vertex { position: corners[1], color: [0.0, 1.0, 0.0] },    // B
        Vertex { position: corners[2], color: [0.0, 0.0, 1.0] },     // C
        Vertex { position: corners[3], color: [1.0, 1.0, 0.0] },      // D;

        //inner square
        Vertex { position: inner_corners[0], color: color_as_array(GREY_COLOR) },     
        Vertex { position: inner_corners[1], color: color_as_array(GREY_COLOR) },   
        Vertex { position: inner_corners[2], color: color_as_array(GREY_COLOR) },    
        Vertex { position: inner_corners[3], color: color_as_array(GREY_COLOR) },
    ];
    let i = *index_start;
    let indices: Vec<u16> = vec![
        //outer square
        i + 0, i + 1, i + 3,    // A, B, D
        i + 3, i + 1, i + 2,    // D, B, C
        //inner square
        i + 4, i + 5, i + 7,
        i + 7, i + 5, i + 6,
    ];
    *index_start += 8;
    return (vertices, indices);
}

fn create_point(p: Point, index_start: &mut u16) -> (Vec<Vertex>, Vec<u16>) {
    let (x, y) = (p.x, p.y);
    const SIZE: f32  = 0.01;
    let color = RED_COLOR;
    let vertices: Vec<Vertex> = vec![
        //outer square
        Vertex { position: [x, y - SIZE], color: color },     // A
        Vertex { position: [x + SIZE, y - SIZE], color: color },    // B
        Vertex { position: [x + SIZE, y], color: color },     // C
        Vertex { position: [x, y], color: color },      // D;
    ];
    let i = *index_start;
    let indices: Vec<u16> = vec![
        //outer square
        i + 0, i + 1, i + 3,    // A, B, D
        i + 3, i + 1, i + 2,    // D, B, C
    ];
    *index_start += 4;
    return (vertices, indices);
}

fn create_line(line: Line, index_start: &mut u16) -> (Vec<Vertex>, Vec<u16>) {
    const W: f32 = 0.005;
    let color: [f32; 3] = BLACK_COLOR;
    let p1 = &line.p1;
    let p2 = &line.p2;

    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    let l = dx.hypot(dy);
    let u = dx * W * 0.5 / l;
    let v = dy * W * 0.5 / l;
    let vertices: Vec<Vertex> = vec![
        Vertex { position: [p1.x + v,  p1.y - u], color },
        Vertex { position: [p1.x - v,  p1.y + u], color },
        Vertex { position: [p2.x - v,  p2.y + u], color },
        Vertex { position: [p2.x + v,  p2.y - u], color },
    ];
    let i = *index_start;
    let indices: Vec<u16> = vec![
        i + 2, i + 1, i + 0,
        i + 2, i + 0, i + 3,
    ];
    *index_start += 4;
    return (vertices, indices);
}

fn init_text<'a>(device: & wgpu::Device, config: &wgpu::SurfaceConfiguration) -> TextBrush<FontRef<'a>> {
    let font: &[u8] = include_bytes!("../fonts/DejaVuSans.ttf");
    return BrushBuilder::using_font_bytes(font).unwrap()
     /* .initial_cache_size((16_384, 16_384))) */ // use this to avoid resizing cache texture
        .build(&device, config.width, config.height, config.format);
}

fn init_vertices(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer, Vec<Vertex>, Vec<u16>) {
    let (v, i) = (Vec::new(), Vec::new());
    let vertex_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&v),
            usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&i),
            usage: wgpu::BufferUsages::INDEX,
        }
    );
    return (vertex_buffer, index_buffer, v, i);
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

pub fn setup_render(device: &wgpu::Device, _surface: &wgpu::Surface<'_>,
    _adapter: &wgpu::Adapter, config: &wgpu::SurfaceConfiguration) -> RenderPipeline {
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
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
            compilation_options: Default::default(),
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
    });
    return render_pipeline;
}

mod math {
    use crate::primitives::Rectangle;

    struct Vec2 {
        pub x: f32,
        pub y: f32,
    }
    struct Mat22 {
        pub col1: Vec2,
        pub col2: Vec2,
    }

    impl Mat22 {
        fn from_angle(angle: f32) -> Self {
            let c = f32::cos(angle);
            let s = f32::sin(angle);
            Self {
                col1: Vec2{x: c, y: s},
                col2: Vec2{x: -s, y: c},
            }
        }
    }

    impl std::ops::Mul<&Vec2> for &Mat22 {
        type Output = Vec2;
        fn mul(self, v: &Vec2) -> Self::Output {
            Vec2{
                x: self.col1.x * v.x + self.col2.x * v.y,
                y: self.col1.y * v.x + self.col2.y * v.y
            }
        }
    }

    impl std::ops::Mul<f32> for &Vec2 {
        type Output = Vec2;
        fn mul(self, other: f32) -> Self::Output {
            Vec2{ x: self.x * other, y: self.y * other }
        }
    } 

    impl std::ops::Add for &Vec2 {
        type Output = Vec2;
        fn add(self, other: Self) -> Self::Output {
            Vec2{ x: self.x + other.x, y: self.y + other.y}
        }
    }

    impl Into<[f32; 2]> for Vec2 {
        fn into(self) -> [f32; 2] {
            [self.x, self.y] 
        }
    }
    
    pub fn get_corners(rect: &Rectangle) -> [[f32; 2]; 4] {
        let r = Mat22::from_angle(rect.rotation);
        let center = Vec2{x: rect.center.x, y: rect.center.y};
        let w = &Vec2{x: rect.width, y: rect.height} * 0.5;

        let left_bot:  [f32; 2] = (&center + &(&r * &Vec2{x: -w.x, y: -w.y})).into();
        let right_bot: [f32; 2] = (&center + &(&r * &Vec2{x:  w.x, y: -w.y})).into();
        let right_top: [f32; 2] = (&center + &(&r * &Vec2{x:  w.x, y:  w.y})).into();
        let left_top:  [f32; 2] = (&center + &(&r * &Vec2{x: -w.x, y:  w.y})).into(); 

        [left_bot, right_bot, right_top, left_top]
    }
}