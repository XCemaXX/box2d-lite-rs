mod buttons;
mod input;
mod letterbox;
mod render;
mod utils;

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{KeyEvent, MouseButton, TouchPhase, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    keyboard::PhysicalKey,
    window::{Window, WindowAttributes, WindowId},
};

use futures::channel::oneshot;
use wasm_bindgen::prelude::*;

use input::InputState;
use letterbox::{Letterbox, Rect};
use physics::PhysicsState;
use render::Render;

const WEBAPP_CANVAS_ID: &str = "target";
const WORLD: Rect = Rect::new(-1.0, -1.0, 2.0, 2.0);
const FPS_PERIOD: f32 = 0.4;

type RenderResult = Result<Render, String>;

enum State {
    Uninit,
    Initializing {
        window: Arc<Window>,
        rx: oneshot::Receiver<RenderResult>,
    },
    Running(Box<Running>),
    Failed,
}

pub struct App {
    // Wake-up only, the data goes through `oneshot`: winit 0.31 drops `user_event(T)`.
    proxy: EventLoopProxy<()>,
    state: State,
}

/// The app after the renderer is ready.
struct Running {
    window: Arc<Window>,
    render: Render,
    letterbox: Letterbox,
    input: InputState,
    physics: PhysicsState,
    last_frame: web_time::Instant,
    fps: FpsCounter,
    surface_per_css: f32,
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
    fn new(proxy: EventLoopProxy<()>) -> Self {
        Self {
            proxy,
            state: State::Uninit,
        }
    }

    fn create_surfaces(&mut self, event_loop: &ActiveEventLoop) {
        // A second `resumed` must not create a second window on the same canvas.
        if !matches!(self.state, State::Uninit) {
            return;
        }
        let window = match window_attributes()
            .and_then(|a| event_loop.create_window(a).map_err(|e| e.to_string()))
        {
            Ok(w) => Arc::new(w),
            Err(e) => {
                log::error!("Create window: {e}");
                self.state = State::Failed;
                return;
            }
        };
        // May still be zero on the web; `Running::new` resizes again.
        let size = window.inner_size();
        let (tx, rx) = oneshot::channel();
        let (proxy, w) = (self.proxy.clone(), window.clone());
        wasm_bindgen_futures::spawn_local(async move {
            let _ = tx.send(Render::new(w, size.width, size.height).await);
            let _ = proxy.send_event(());
        });
        self.state = State::Initializing { window, rx };
    }

    fn on_wake(&mut self) {
        let State::Initializing { rx, .. } = &mut self.state else {
            return;
        };
        let result = match rx.try_recv() {
            Ok(None) => return,
            Ok(Some(r)) => r,
            Err(oneshot::Canceled) => Err("render init task dropped".into()),
        };
        let State::Initializing { window, .. } = std::mem::replace(&mut self.state, State::Failed)
        else {
            unreachable!()
        };
        match result {
            Ok(render) => {
                let running = Running::new(window, render);
                running.window.request_redraw();
                self.state = State::Running(Box::new(running));
            }
            Err(e) => log::error!("Render init failed: {e}"),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.create_surfaces(event_loop);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, (): ()) {
        self.on_wake();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let WindowEvent::CloseRequested = event {
            log::info!("Close requested. Exiting...");
            event_loop.exit();
            return;
        }
        // Events before `Running` are dropped: `on_wake` reads the current size.
        let State::Running(app) = &mut self.state else {
            return;
        };
        app.handle(event);
    }
}

fn window_attributes() -> Result<WindowAttributes, String> {
    use winit::platform::web::WindowAttributesExtWebSys;
    let canvas = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(WEBAPP_CANVAS_ID))
        .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        .ok_or_else(|| format!("no <canvas id={WEBAPP_CANVAS_ID}>"))?;
    Ok(Window::default_attributes()
        .with_canvas(Some(canvas))
        // Keep browser keys (F5, F11, F12) working; CSS already blocks scroll and touch gestures.
        .with_prevent_default(false))
}

impl Running {
    fn new(window: Arc<Window>, mut render: Render) -> Self {
        // Resizes during init were dropped: take the size now.
        let size = window.inner_size();
        let (w, h) = render.resize(size.width, size.height);
        let mut letterbox = Letterbox::new(WORLD, true);
        letterbox.resize(w as f32, h as f32);
        let surface_per_css = surface_per_css(&window, w);
        Self {
            window,
            render,
            letterbox,
            input: InputState::default(),
            physics: PhysicsState::with_jitter(|| rand::random_range(-1.0..=1.0)),
            last_frame: web_time::Instant::now(),
            fps: FpsCounter::default(),
            surface_per_css,
        }
    }

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
                self.surface_per_css = surface_per_css(&self.window, w);
            }
            WindowEvent::RedrawRequested => {
                self.redraw();
                // On the web this is the next requestAnimationFrame.
                self.window.request_redraw();
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.input.update_cursor_buttons(state.is_pressed());
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.update_pointer(position);
            }
            WindowEvent::Touch(t) => {
                self.update_pointer(t.location);
                match t.phase {
                    TouchPhase::Started => self.input.update_cursor_buttons(true),
                    TouchPhase::Ended => self.input.update_cursor_buttons(false),
                    _ => {}
                }
            }
            _ => (),
        }
    }

    fn update_pointer(&mut self, pos: PhysicalPosition<f64>) {
        // Workaround for the winit web pointer scale bug, see `surface_per_css`.
        let k = self.surface_per_css / self.window.scale_factor() as f32;
        let (x, y) = (pos.x as f32 * k, pos.y as f32 * k);
        self.input.update_cursor_pos(self.letterbox.to_world(x, y));
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
        self.render
            .render(self.letterbox.field(), self.surface_per_css, &text);
    }
}

/// Surface pixels per CSS pixel, measured from the canvas.
///
/// Workaround for a winit 0.30 web bug: pointer positions are CSS px * devicePixelRatio,
/// but the canvas buffer is sized from `devicePixelContentBoxSize`. On some phones the two
/// ratios differ, and touches drift away from the finger towards the bottom right.
/// Remove when winit gives pointer positions in canvas buffer pixels.
fn surface_per_css(window: &Window, surface_w: u32) -> f32 {
    use winit::platform::web::WindowExtWebSys;
    match window.canvas().map(|c| c.client_width()) {
        Some(css_w) if css_w > 0 => surface_w as f32 / css_w as f32,
        _ => window.scale_factor() as f32,
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
                if let Err(e) = physics_state.set_scene(scene) {
                    log::warn!("{e}");
                }
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
    // Before the event loop: an early panic must have readable text too.
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    console_log::init().expect("Failed to initialize logger!");

    let event_loop = match EventLoop::with_user_event().build() {
        Ok(el) => el,
        Err(e) => {
            log::error!("Create event loop: {e}");
            return;
        }
    };
    let app = App::new(event_loop.create_proxy());
    // `run_app` on the web leaves the loop by throwing a JS exception.
    use winit::platform::web::EventLoopExtWebSys;
    event_loop.spawn_app(app);
}
