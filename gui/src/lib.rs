mod utils;
mod render;
mod input;
mod primitives;
mod physics;

use wasm_bindgen::prelude::*;
use winit::{
    event::{Event, WindowEvent, MouseButton, KeyEvent}, 
    event_loop::EventLoop, window::{Window, WindowBuilder}
};
use wgpu::web_sys;
use winit::error::OsError;
use winit::keyboard::PhysicalKey;

use render::Render;
use input::InputState;
use physics::PhysicsState;

const WEBAPP_CANVAS_ID: &str = "target";

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut render_state = Render::new(&window).await;    
    let mut input_state = InputState::default();
    let mut physics_state = PhysicsState::new(0);

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
                    }
                    WindowEvent::RedrawRequested => {
                        let dt = last_time.elapsed().as_secs_f32();
                        last_time = instant::Instant::now();
                        window.request_redraw();

                        input_events_to_physics(&mut input_state, &mut physics_state);
                        physics_state.step(dt);
                        let rectangles = physics_state.get_rectangles();
                        let collide_points = physics_state.get_collide_points();
                        let joint_lines = physics_state.get_joint_lines();
                        render_state.text = format!("{}\nfps: {:.3}\n{}", 
                            input_state, 1.0 / dt, physics_state.get_scene_name());

                        render_state.update_frame(rectangles, collide_points, joint_lines);
                        render_state.render();
                    },
                    WindowEvent::CursorEntered{..} => {
                        input_state.update_mouse_inside(true);
                    },
                    WindowEvent::CursorLeft { .. } => {
                        input_state.update_mouse_inside(false);
                    },
                    WindowEvent::MouseInput { state, button, .. } => {
                        if button == MouseButton::Left {
                            input_state.update_mouse_buttons(state.is_pressed());
                        }
                    },
                    WindowEvent::CursorMoved { position, .. } => {
                        let (x, y) = coords_to_render(position.x, position.y, window.inner_size());
                        input_state.update_mouse_pos(x, y);
                    },
                    WindowEvent::KeyboardInput { event: KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(key),
                        ..
                    }, .. } => {
                        input_state.update_keyboard(state.is_pressed(), key);
                    },
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

fn input_events_to_physics(input_state: &mut InputState, physics_state: &mut PhysicsState) {
    while let Some(event) = input_state.pop_event() {
        match event {
            input::Event::Restart => { physics_state.restart(); },
            input::Event::CreateBox(x, y) => { physics_state.add_rectangle(x, y); },
            input::Event::RunScene(scene) => { *physics_state = PhysicsState::new(scene); }
        }
    }
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
    //alert("Hello, gui!");
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init().expect("could not initialize logger");

    let event_loop = EventLoop::new().unwrap();
    let window = create_window(&event_loop).unwrap();

    wasm_bindgen_futures::spawn_local(run(event_loop, window));
}