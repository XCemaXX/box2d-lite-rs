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
    pending_render: Option<Receiver<Render<'static>>>,
    running: Option<Running>,
    // Resize can come before the renderer is ready: keep the size for it.
    last_size: (u32, u32),
}

/// The app after the renderer is ready.
struct Running {
    window: Arc<Window>,
    render: Render<'static>,
    letterbox: Letterbox,
    input: InputState,
    physics: PhysicsState,
    last_frame: web_time::Instant,
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

impl App {
    fn try_start(&mut self) {
        let (Some(receiver), Some(window)) = (self.pending_render.as_mut(), self.window.as_ref())
        else {
            return;
        };
        let Ok(Some(mut render)) = receiver.try_recv() else {
            return;
        };
        self.pending_render = None;
        let (w, h) = render.resize(self.last_size.0, self.last_size.1);
        let mut letterbox = Letterbox::new(WORLD, true);
        letterbox.resize(w as f32, h as f32);
        self.running = Some(Running {
            window: window.clone(),
            render,
            letterbox,
            input: InputState::default(),
            physics: PhysicsState::new(0),
            last_frame: web_time::Instant::now(),
            fps: FpsCounter::default(),
        });
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
                self.pending_render = Some(receiver);
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init().expect("Failed to initialize logger!");
                log::info!("Canvas dimensions: ({canvas_width} x {canvas_height})");
                wasm_bindgen_futures::spawn_local(async move {
                    let render = Render::new(window_handle, canvas_width, canvas_height).await;
                    if sender.send(render).is_err() {
                        log::error!("Failed to create and send render!");
                    }
                });
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let WindowEvent::Resized(PhysicalSize { width, height }) = event {
            self.last_size = (width, height);
        }
        self.try_start();
        let Some(app) = self.running.as_mut() else {
            return;
        };
        if let WindowEvent::CloseRequested = event {
            log::info!("Close requested. Exiting...");
            event_loop.exit();
            return;
        }
        app.handle(event);
        app.window.request_redraw();
    }
}

impl Running {
    fn handle(&mut self, event: WindowEvent) {
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
                self.input.update_keyboard(state.is_pressed(), key);
            }
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                let (w, h) = self.render.resize(width, height);
                self.letterbox.resize(w as f32, h as f32);
            }
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.input.update_cursor_buttons(state.is_pressed());
            }
            // Positions are in surface pixels, the same space as the letterbox field.
            WindowEvent::CursorMoved { position, .. } => {
                let pos = self
                    .letterbox
                    .to_world(position.x as f32, position.y as f32);
                self.input.update_cursor_pos(pos);
            }
            WindowEvent::Touch(t) => {
                let pos = self
                    .letterbox
                    .to_world(t.location.x as f32, t.location.y as f32);
                self.input.update_cursor_pos(pos);
                match t.phase {
                    TouchPhase::Started => self.input.update_cursor_buttons(true),
                    TouchPhase::Ended => self.input.update_cursor_buttons(false),
                    _ => {}
                }
            }
            _ => (),
        }
    }

    fn redraw(&mut self) {
        let dt = self.last_frame.elapsed().as_secs_f32();
        self.last_frame = web_time::Instant::now();
        self.fps.add_frame(dt);

        input_events_to_physics(&mut self.input, &mut self.physics);
        self.physics.step(dt);

        let controls = "Controls: 1-9 scenes; Space: restart; P, N - prev, next scene;
               Click - add box";
        let (fps, physics) = (self.fps.value, &self.physics);
        let text = if cfg!(debug_assertions) {
            format!("{}\nfps: {fps:.1}\n{physics}\n{controls}", self.input)
        } else {
            format!("fps: {fps:.1}\n{physics}\n{controls}")
        };

        self.render.update_frame(&self.letterbox, physics);
        let scale_factor = self.window.scale_factor() as f32;
        self.render
            .render(self.letterbox.field(), scale_factor, &text);
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
