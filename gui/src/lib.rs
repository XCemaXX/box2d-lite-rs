mod game;
mod input;
mod layout;
mod letterbox;
mod render;
mod scene;

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyEvent, MouseButton, TouchPhase, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    keyboard::PhysicalKey,
    window::{Window, WindowAttributes, WindowId},
};

use futures::channel::oneshot;
use wasm_bindgen::prelude::*;

use game::Game;
use layout::WORLD;
use letterbox::Letterbox;
use render::{Mesh, Render};

const WEBAPP_CANVAS_ID: &str = "target";

enum State {
    Uninit,
    Initializing {
        window: Arc<Window>,
        rx: oneshot::Receiver<Result<Render, String>>,
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
    mesh: Mesh,
    letterbox: Letterbox,
    surface_per_css: f32,
    // Raw winit position: the world position changes on resize.
    last_pointer: Option<PhysicalPosition<f64>>,
    last_frame: web_time::Instant,
    game: Game,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
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
        // Often zero here: the canvas is not measured yet. `Running::new` resizes again.
        let size = window.inner_size();
        let (tx, rx) = oneshot::channel();
        let (proxy, w) = (self.proxy.clone(), window.clone());
        wasm_bindgen_futures::spawn_local(async move {
            let _ = tx.send(Render::new(w, size.width, size.height).await);
            let _ = proxy.send_event(());
        });
        self.state = State::Initializing { window, rx };
    }

    // The render init task is done.
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, (): ()) {
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
        // Events before `Running` are dropped: `Running::new` reads the current size.
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
    fn new(window: Arc<Window>, render: Render) -> Self {
        let mut running = Self {
            window,
            render,
            mesh: Mesh::default(),
            letterbox: Letterbox::new(WORLD),
            surface_per_css: 1.0,
            last_pointer: None,
            last_frame: web_time::Instant::now(),
            game: Game::new(),
        };
        // Resizes during init were dropped: take the size now.
        running.resize(running.window.inner_size());
        running
    }

    /// The surface, letterbox and pointer scale must change together.
    fn resize(&mut self, size: PhysicalSize<u32>) {
        let (w, h) = self.render.resize(size.width, size.height);
        self.letterbox.resize(w as f32, h as f32);
        self.surface_per_css = input::surface_per_css(&self.window, w);
        if let Some(pos) = self.last_pointer {
            self.update_pointer(pos);
        }
    }

    fn handle(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(key),
                        repeat: false,
                        ..
                    },
                ..
            } => {
                if let Some(e) = input::key_event(key) {
                    self.game.apply(e);
                }
            }
            WindowEvent::Resized(size) => self.resize(size),
            WindowEvent::RedrawRequested => {
                self.redraw();
                // On the web this is the next requestAnimationFrame.
                self.window.request_redraw();
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.game.pointer_pressed();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.update_pointer(position);
            }
            WindowEvent::Touch(t) => {
                self.update_pointer(t.location);
                if t.phase == TouchPhase::Started {
                    self.game.pointer_pressed();
                }
            }
            _ => (),
        }
    }

    fn update_pointer(&mut self, pos: PhysicalPosition<f64>) {
        self.last_pointer = Some(pos);
        let [x, y] = input::to_surface(&self.window, self.surface_per_css, pos);
        self.game.pointer_moved(self.letterbox.to_world(x, y));
    }

    fn redraw(&mut self) {
        let dt = self.last_frame.elapsed().as_secs_f32();
        self.last_frame = web_time::Instant::now();
        self.game.update(dt);

        scene::build(&mut self.mesh, &self.letterbox, self.game.physics());
        let field = self.letterbox.field();
        self.render.draw(
            &self.mesh,
            scene::BARS,
            field,
            self.surface_per_css,
            &self.game.hud(),
        );
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
    let app = App {
        proxy: event_loop.create_proxy(),
        state: State::Uninit,
    };
    // `run_app` on the web leaves the loop by throwing a JS exception.
    use winit::platform::web::EventLoopExtWebSys;
    event_loop.spawn_app(app);
}
