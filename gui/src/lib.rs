mod buttons;
mod input;
mod render;
mod utils;

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{KeyEvent, MouseButton, TouchPhase, WindowEvent},
    keyboard::PhysicalKey,
    window::Window,
};

use futures::channel::oneshot::Receiver;
use wasm_bindgen::prelude::*;

use input::InputState;
use physics::PhysicsState;
use render::Render;

const WEBAPP_CANVAS_ID: &str = "target";

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    render: Option<Render<'static>>,
    last_render_time: Option<instant::Instant>,
    renderer_receiver: Option<Receiver<Render<'static>>>,
    last_size: (u32, u32),
    input_state: Option<InputState>,
    physics_state: Option<PhysicsState>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let mut attributes = Window::default_attributes();

        use winit::platform::web::WindowAttributesExtWebSys;
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(WEBAPP_CANVAS_ID)
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        let canvas_width = canvas.width();
        let canvas_height = canvas.height();
        self.last_size = (canvas_width, canvas_height);
        attributes = attributes.with_canvas(Some(canvas));

        if let Ok(window) = event_loop.create_window(attributes) {
            let first_window_handle = self.window.is_none();
            let window_handle = Arc::new(window);
            self.window = Some(window_handle.clone());
            if first_window_handle {
                let (sender, receiver) = futures::channel::oneshot::channel();
                self.renderer_receiver = Some(receiver);
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init().expect("Failed to initialize logger!");
                log::info!("Canvas dimensions: ({canvas_width} x {canvas_height})");
                wasm_bindgen_futures::spawn_local(async move {
                    let render =
                        Render::new(window_handle.clone(), canvas_width, canvas_height).await;
                    if sender.send(render).is_err() {
                        log::error!("Failed to create and send render!");
                    }
                });
                self.last_render_time = Some(instant::Instant::now());
                self.input_state = Some(InputState::default());
                self.physics_state = Some(PhysicsState::new(0));
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let Some(mut receiver) = self.renderer_receiver.take() {
            if let Ok(Some(render)) = receiver.try_recv() {
                self.render = Some(render);
            } else {
                self.renderer_receiver = Some(receiver);
            }
        }

        let (
            Some(render_state),
            Some(window),
            Some(last_render_time),
            Some(input_state),
            Some(physics_state),
        ) = (
            self.render.as_mut(),
            self.window.as_ref(),
            self.last_render_time.as_mut(),
            self.input_state.as_mut(),
            self.physics_state.as_mut(),
        )
        else {
            return;
        };

        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(key),
                        ..
                    },
                ..
            } => {
                input_state.update_keyboard(state.is_pressed(), key);
            }
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                let (width, height) = ((width).max(1), (height).max(1));
                log::info!("Resizing renderer surface to: ({width}, {height})");
                render_state.resize(width, height);
                self.last_size = (width, height);
            }
            WindowEvent::CloseRequested => {
                log::info!("Close requested. Exiting...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let dt = last_render_time.elapsed().as_secs_f32();
                *last_render_time = instant::Instant::now();

                input_events_to_physics(input_state, physics_state);
                physics_state.step(dt);
                let rectangles = physics_state.get_rectangles();
                let collide_points = physics_state.get_collide_points();
                let joint_lines = physics_state.get_joint_lines();

                let controls_text = "Controls: 1-9 scenes; Space: restart; P, N - prev, next scene;
               Click - add box";
                if cfg!(debug_assertions) {
                    render_state.text = format!(
                        "{}\nfps: {:.3}\n{}\n{}",
                        input_state,
                        1.0 / dt,
                        physics_state,
                        controls_text
                    );
                } else {
                    render_state.text =
                        format!("fps: {:.3}\n{}\n{}", 1.0 / dt, physics_state, controls_text);
                }

                render_state.update_frame(rectangles, collide_points, joint_lines);
                render_state.render();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left {
                    input_state.update_cursor_buttons(state.is_pressed());
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = mouse_coords_to_render(position.x, position.y, window.inner_size());
                input_state.update_cursor_pos(x, y);
            }
            WindowEvent::Touch(t) => {
                let (x, y) = touch_coords_to_render(t.location.x, t.location.y, window);
                input_state.update_cursor_pos(x, y);
                match t.phase {
                    TouchPhase::Started => {
                        input_state.update_cursor_buttons(true);
                    }
                    TouchPhase::Ended => {
                        input_state.update_cursor_buttons(false);
                    }
                    _ => {}
                };
            }
            _ => (),
        }
        window.request_redraw();
    }
}

fn touch_coords_to_render(x: f64, y: f64, window: &Window) -> (f32, f32) {
    // work around
    // check this https://github.com/mvlabat/bevy_egui/issues/104
    // should be fixed https://github.com/rust-windowing/winit/pull/2188
    // but doesn't work
    let s = window.scale_factor();
    if let Some(o) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(WEBAPP_CANVAS_ID))
        .map(|element| element.get_bounding_client_rect())
        .map(|rect| (rect.width(), rect.height()))
    {
        let x_normalized = (x / s) / o.0;
        let y_normalized = (y / s) / o.0;

        (
            (x_normalized * 2.0 - 1.0) as f32,
            -(y_normalized * 2.0 - 1.0) as f32,
        )
    } else {
        mouse_coords_to_render(x, y, window.inner_size())
    }
}

fn mouse_coords_to_render(
    x: f64,
    y: f64,
    window_size: winit::dpi::PhysicalSize<u32>,
) -> (f32, f32) {
    let xr = (x * 2.0 / window_size.width as f64) - 1.0;
    let yr = ((window_size.height as f64 - y) * 2.0 / window_size.height as f64) - 1.0;
    return (xr as f32, yr as f32);
}

fn _len_to_render(
    diff_x: f32,
    diff_y: f32,
    window_size: winit::dpi::PhysicalSize<u32>,
) -> (f32, f32) {
    let xr = diff_x * 2.0 / window_size.width as f32;
    let yr = diff_y * 2.0 / window_size.height as f32;
    return (xr as f32, yr as f32);
}

fn input_events_to_physics(input_state: &mut InputState, physics_state: &mut PhysicsState) {
    use input::Event::*;
    while let Some(event) = input_state.pop_event() {
        match event {
            Restart => {
                physics_state.restart();
            }
            CreateBox(x, y) => {
                physics_state.add_rectangle(x, y);
            }
            RunScene(scene) => {
                *physics_state = PhysicsState::new(scene);
            }
            ChangeToNextScene => {
                physics_state.change_to_next_scene();
            }
            ChangeToPrevScene => {
                physics_state.change_to_prev_scene();
            }
        }
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    //alert("Hello, gui!");

    let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
