mod buttons;
mod input;
mod letterbox;
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
use letterbox::{Letterbox, Rect};
use physics::PhysicsState;
use render::Render;

const WEBAPP_CANVAS_ID: &str = "target";
const WORLD: Rect = Rect::new(-1.0, -1.0, 2.0, 2.0);
const FPS_PERIOD: f32 = 0.4;

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
    last_render_time: Option<web_time::Instant>,
    renderer_receiver: Option<Receiver<Render<'static>>>,
    last_size: (u32, u32),
    input_state: Option<InputState>,
    physics_state: Option<PhysicsState>,
    letterbox: Option<Letterbox>,
    fps: FpsCounter,
}

/// Average fps, updated once per `FPS_PERIOD` so the number is readable.
#[derive(Default)]
struct FpsCounter {
    value: f32,
    frames: u32,
    time: f32,
}

impl FpsCounter {
    fn add_frame(&mut self, dt: f32) {
        self.frames += 1;
        self.time += dt;
        if self.time >= FPS_PERIOD {
            self.value = self.frames as f32 / self.time;
            self.frames = 0;
            self.time = 0.0;
        }
    }
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
                self.last_render_time = Some(web_time::Instant::now());
                self.input_state = Some(InputState::default());
                self.physics_state = Some(PhysicsState::new(0));
                self.letterbox = Some(Letterbox::new(WORLD, true));
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        // Resize can come before the renderer is ready: keep the size for it.
        if let WindowEvent::Resized(PhysicalSize { width, height }) = event {
            self.last_size = (width, height);
        }
        if let Some(mut receiver) = self.renderer_receiver.take() {
            if let Ok(Some(mut render)) = receiver.try_recv() {
                let (w, h) = render.resize(self.last_size.0, self.last_size.1);
                if let Some(letterbox) = self.letterbox.as_mut() {
                    letterbox.resize(w as f32, h as f32);
                }
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
            Some(letterbox),
        ) = (
            self.render.as_mut(),
            self.window.as_ref(),
            self.last_render_time.as_mut(),
            self.input_state.as_mut(),
            self.physics_state.as_mut(),
            self.letterbox.as_mut(),
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
                let (w, h) = render_state.resize(width, height);
                letterbox.resize(w as f32, h as f32);
            }
            WindowEvent::CloseRequested => {
                log::info!("Close requested. Exiting...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let dt = last_render_time.elapsed().as_secs_f32();
                *last_render_time = web_time::Instant::now();
                self.fps.add_frame(dt);
                let fps = self.fps.value;

                input_events_to_physics(input_state, physics_state);
                physics_state.step(dt);
                let rectangles = physics_state.get_rectangles();
                let collide_points = physics_state.get_collide_points();
                let joint_lines = physics_state.get_joint_lines();

                let controls_text = "Controls: 1-9 scenes; Space: restart; P, N - prev, next scene;
               Click - add box";
                if cfg!(debug_assertions) {
                    render_state.text = format!(
                        "{}\nfps: {:.1}\n{}\n{}",
                        input_state, fps, physics_state, controls_text
                    );
                } else {
                    render_state.text =
                        format!("fps: {fps:.1}\n{}\n{}", physics_state, controls_text);
                }

                render_state.update_frame(letterbox, rectangles, collide_points, joint_lines);
                render_state.render(letterbox.field(), window.scale_factor() as f32);
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                input_state.update_cursor_buttons(state.is_pressed());
            }
            // Positions are in surface pixels, the same space as the letterbox field.
            WindowEvent::CursorMoved { position, .. } => {
                input_state
                    .update_cursor_pos(letterbox.to_world(position.x as f32, position.y as f32));
            }
            WindowEvent::Touch(t) => {
                let pos = letterbox.to_world(t.location.x as f32, t.location.y as f32);
                input_state.update_cursor_pos(pos);
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
