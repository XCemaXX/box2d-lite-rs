mod utils;

use wasm_bindgen::prelude::*;
use winit::{
    event::{Event, WindowEvent, MouseButton}, event_loop::EventLoop, window::{Window, WindowBuilder, CursorIcon}
};
use wgpu::web_sys;
use std::borrow::Cow;
use winit::error::OsError;
use wgpu_text::{glyph_brush::{Section as TextSection, Text}, BrushBuilder};

const GLOBAL_LOG_FILTER: log::LevelFilter = log::LevelFilter::Info;
const WEBAPP_CANVAS_ID: &str = "target";

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

struct MouseState {
    pub is_cursor_inside: bool,
    pub is_left_pressed: bool,
    pub x: f64,
    pub y: f64,
}

impl MouseState {
    pub fn new() -> Self {
        MouseState {
            is_cursor_inside: false,
            is_left_pressed: false,
            x: 0.0,
            y: 0.0,
        }
    }

    pub fn to_string(&self) -> String {
        if self.is_left_pressed {
            format!("PRESSED. Pos: {} {}", self.x.round(), self.y.round())
        } else if self.is_cursor_inside {
            format!("IN BOX. Pos: {} {}", self.x.round(), self.y.round())
        } else {
            format!("OUT BOX")
        }
    }
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut size = window.inner_size();
    size.width = size.width.max(500);
    size.height = size.height.max(500);

    let instance = wgpu::Instance::default();

    //window.set_cursor_icon(CursorIcon::Grab);
    let surface = instance.create_surface(&window).unwrap();
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

    // Load the shaders from disk
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let mut config = surface
        .get_default_config(&adapter, size.width, size.height)
        .unwrap();
    surface.configure(&device, &config);

    let font: &[u8] = include_bytes!("../fonts/DejaVuSans.ttf");
    let mut text_brush = BrushBuilder::using_font_bytes(font).unwrap()
    /* .initial_cache_size((16_384, 16_384))) */ // use this to avoid resizing cache texture
        .build(&device, config.width, config.height, config.format);
    let mut label_text = "START".to_string();
    
    let mut mouse_state = MouseState::new();

    log("Start event loop");
    let window = &window;
    event_loop
        .run(move |event, target| {
            let _ = (&instance, &adapter, &shader, &pipeline_layout);

            if let Event::WindowEvent {
                window_id: _,
                event,
            } = event
            {
                match event {
                    WindowEvent::Resized(new_size) => {
                        text_brush.resize_view(config.width as f32, config.height as f32, &queue);
                        // Reconfigure the surface with the new size
                        //config.width = 500; // new_size.width.max(1);
                        //config.height = 500; // new_size.height.max(1);
                        //surface.configure(&device, &config);
                        // On macos the window needs to be redrawn manually after resizing
                        //window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        let frame = surface
                            .get_current_texture()
                            .expect("Failed to acquire next swap chain texture");
                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                                            load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                            store: wgpu::StoreOp::Store,
                                        },
                                    })],
                                    depth_stencil_attachment: None,
                                    timestamp_writes: None,
                                    occlusion_query_set: None,
                                });
                            rpass.set_pipeline(&render_pipeline);
                            rpass.draw(0..3, 0..1);
                            
                            label_text = mouse_state.to_string();
                            let text_section = TextSection::default().add_text(Text::new(&label_text));
                            text_brush.queue(&device, &queue, vec![&text_section]).unwrap();
                            text_brush.draw(&mut rpass);
                        }

                        queue.submit(Some(encoder.finish()));
                        frame.present();
                        window.request_redraw();
                    },
                    WindowEvent::CursorEntered{..} => {
                        mouse_state.is_cursor_inside = true;
                    },
                    WindowEvent::CursorLeft { .. } => {
                        mouse_state.is_cursor_inside = false;
                    },
                    WindowEvent::MouseInput { state, button, .. } => {
                        if button == MouseButton::Left {
                            mouse_state.is_left_pressed = state.is_pressed();
                        }
                    },
                    WindowEvent::CursorMoved { position, .. } => {
                        mouse_state.x = position.x;
                        mouse_state.y = position.y;
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    _ => {}
                };
            }
        })
        .unwrap();
}

fn create_window<T>(event_loop: &EventLoop<T>) -> Result<Window, OsError> {
    use wasm_bindgen::JsCast;
    use winit::platform::web::WindowBuilderExtWebSys;
    let dom_window = web_sys::window().unwrap();
    let dom_document = dom_window.document().unwrap();
    let dom_canvas = dom_document.get_element_by_id(WEBAPP_CANVAS_ID).unwrap();
    let canvas = dom_canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok();
    WindowBuilder::default()
        .with_canvas(canvas)
        .build(event_loop)
}

#[wasm_bindgen(start)]
pub fn start() {
    //alert("Hello, hello-wasm!");
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init().expect("could not initialize logger");

    let event_loop = EventLoop::new().unwrap();
    let window = create_window(&event_loop).unwrap();

    wasm_bindgen_futures::spawn_local(run(event_loop, window));
}