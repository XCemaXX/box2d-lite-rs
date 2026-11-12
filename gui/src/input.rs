//! winit input to game events and surface pixels.
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, TouchPhase, WindowEvent},
    keyboard::KeyCode,
    window::Window,
};

use crate::game::Event;

pub fn key_event(key: KeyCode) -> Option<Event> {
    Some(match key {
        KeyCode::Space => Event::Restart,
        KeyCode::KeyN => Event::NextScene,
        KeyCode::KeyP => Event::PrevScene,
        // Digit1..=Digit9 are declared in order in winit.
        digit => match (digit as usize).wrapping_sub(KeyCode::Digit1 as usize) {
            n @ 0..=8 => Event::RunScene(n),
            _ => return None,
        },
    })
}

/// Pointer input in surface pixels.
pub enum Pointer {
    Moved([f32; 2]),
    /// The left button or the first finger.
    Pressed([f32; 2]),
}

/// Keeps what an event alone does not have: the last position and the finger in use.
#[derive(Default)]
pub struct PointerMapper {
    // Raw winit position: surface pixels change when the scale changes.
    last_raw: Option<PhysicalPosition<f64>>,
    // The finger that acts as the pointer; other fingers are ignored.
    touch: Option<u64>,
}

impl PointerMapper {
    /// The last position in the current scale: send it again after a resize.
    pub fn last(&self, window: &Window, surface_per_css: f32) -> Option<[f32; 2]> {
        self.last_raw
            .map(|raw| to_surface(window, surface_per_css, raw))
    }

    pub fn map(
        &mut self,
        window: &Window,
        event: &WindowEvent,
        surface_per_css: f32,
    ) -> Option<Pointer> {
        let to_surface = |pos| to_surface(window, surface_per_css, pos);
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.last_raw = Some(*position);
                Some(Pointer::Moved(to_surface(*position)))
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => Some(Pointer::Pressed(to_surface(self.last_raw?))),
            WindowEvent::Touch(t) => {
                match t.phase {
                    TouchPhase::Started if self.touch.is_none() => self.touch = Some(t.id),
                    _ if self.touch != Some(t.id) => return None,
                    TouchPhase::Ended | TouchPhase::Cancelled => self.touch = None,
                    _ => {}
                }
                self.last_raw = Some(t.location);
                let pos = to_surface(t.location);
                Some(match t.phase {
                    TouchPhase::Started => Pointer::Pressed(pos),
                    _ => Pointer::Moved(pos),
                })
            }
            _ => None,
        }
    }
}

fn to_surface(window: &Window, surface_per_css: f32, pos: PhysicalPosition<f64>) -> [f32; 2] {
    let k = surface_per_css / window.scale_factor() as f32;
    [pos.x as f32 * k, pos.y as f32 * k]
}

/// Surface pixels per CSS pixel, measured from the canvas.
///
/// Workaround for a winit 0.30 web bug: pointer positions are CSS px * devicePixelRatio,
/// but the canvas buffer is sized from `devicePixelContentBoxSize`. On some phones the two
/// ratios differ, and touches drift away from the finger towards the bottom right.
/// Remove with `to_surface` when winit gives pointer positions in canvas buffer pixels.
pub fn surface_per_css(window: &Window, surface_w: u32) -> f32 {
    use winit::platform::web::WindowExtWebSys;
    match window.canvas().map(|c| c.client_width()) {
        Some(css_w) if css_w > 0 => surface_w as f32 / css_w as f32,
        _ => window.scale_factor() as f32,
    }
}
