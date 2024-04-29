mod utils;
mod render;
mod mouse;

use wasm_bindgen::prelude::*;
use winit::{
    event::{Event, WindowEvent, MouseButton}, event_loop::EventLoop, window::{Window, WindowBuilder}
};
use wgpu::web_sys;
use winit::error::OsError;

use render::Render;
use mouse::MouseState;

const GLOBAL_LOG_FILTER: log::LevelFilter = log::LevelFilter::Info;
const WEBAPP_CANVAS_ID: &str = "target";

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
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

async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut render_state = Render::new(&window).await;    
    let mut mouse_state = MouseState::new();

    log("Start event loop");
    let window = &window;
    
    event_loop
        .run(move |event, target| {
            //let _ = (&instance, &adapter, &shader, &pipeline_layout);

            if let Event::WindowEvent {
                window_id: _,
                event,
            } = event
            {
                match event {
                    WindowEvent::Resized(new_size) => {
                        //text_brush.resize_view(config.width as f32, config.height as f32, &queue);
                        // Reconfigure the surface with the new size
                        //config.width = 500; // new_size.width.max(1);
                        //config.height = 500; // new_size.height.max(1);
                        //surface.configure(&device, &config);
                        // On macos the window needs to be redrawn manually after resizing
                        //window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        render_state.text = mouse_state.to_string();
                        render_state.render();

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