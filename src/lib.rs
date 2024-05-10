mod utils;
mod render;
mod mouse;
mod box2d;
mod primitives;
mod physics;

use wasm_bindgen::prelude::*;
use winit::{
    event::{Event, WindowEvent, MouseButton}, event_loop::EventLoop, window::{Window, WindowBuilder}
};
use wgpu::web_sys;
use winit::error::OsError;

use render::Render;
use mouse::MouseState;
use physics::WorldState;

const WEBAPP_CANVAS_ID: &str = "target";

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut render_state = Render::new(&window).await;    
    let mut mouse_state = MouseState::new();
    let mut world_state = WorldState::new();

    log("Start event loop");
    let window = &window;
    let mut last_time = instant::Instant::now();

    event_loop
        .run(move |event, target| {
            if let Event::WindowEvent {
                window_id: _,
                event,
            } = event
            {
                match event {
                    WindowEvent::Resized(_new_size) => {
                        //text_brush.resize_view(config.width as f32, config.height as f32, &queue);
                        // Reconfigure the surface with the new size
                        //config.width = 500; // new_size.width.max(1);
                        //config.height = 500; // new_size.height.max(1);
                        //surface.configure(&device, &config);
                        // On macos the window needs to be redrawn manually after resizing
                        //window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        let dt = last_time.elapsed().as_secs_f32();
                        last_time = instant::Instant::now();
                        window.request_redraw();

                        world_state.step(dt);
                        let rectangles = world_state.get_rectangles();
                        let collide_points = world_state.get_collide_points();

                        let mut s = String::new();
                        for r in &rectangles {
                            s = format!("{}\n{:?}", s, r);
                        }
                        for cp in &collide_points {
                            s = format!("{}\n{:?}", s, cp);
                        }
                        render_state.text = format!("{}\nfps: {:.3}\n{}", mouse_state.to_string(), 1.0 / dt, s);

                        render_state.update_frame(rectangles, collide_points);
                        render_state.render();
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
                        let (x, y) = coords_to_render(position.x, position.y, window.inner_size());
                        mouse_state.set_pos(x, y);
                        if mouse_state.is_left_pressed {
                            world_state = WorldState::new();
                        }
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    _ => {}
                };
            }
        })
        .unwrap();
}

fn coords_to_render(x: f64, y: f64, window_size: winit::dpi::PhysicalSize<u32>) -> (f32, f32) {
    let xr = (x * 2.0 / window_size.width as f64) - 1.0;
    let yr = ((window_size.height as f64 - y) * 2.0 /window_size.height as f64) - 1.0;
    return (xr as f32, yr as f32);
}

fn _len_to_render(diff_x: f32, diff_y: f32, window_size: winit::dpi::PhysicalSize<u32>) -> (f32, f32) {
    let xr = diff_x * 2.0 / window_size.width as f32;
    let yr = diff_y * 2.0 / window_size.height as f32;
    return (xr as f32, yr as f32);
}

fn create_window<T>(event_loop: &EventLoop<T>) -> Result<Window, OsError> {
    use winit::platform::web::WindowBuilderExtWebSys;
    let dom_window = web_sys::window().unwrap();
    let dom_document = dom_window.document().unwrap();
    let dom_canvas = dom_document.get_element_by_id(WEBAPP_CANVAS_ID).unwrap();
    let canvas = dom_canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok();
    WindowBuilder::default().with_canvas(canvas).build(event_loop)
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