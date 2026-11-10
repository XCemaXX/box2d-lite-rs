//! winit input to game events and world coordinates.
use winit::{dpi::PhysicalPosition, keyboard::KeyCode, window::Window};

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

/// Pointer position from winit to surface pixels.
pub fn to_surface(window: &Window, surface_per_css: f32, pos: PhysicalPosition<f64>) -> [f32; 2] {
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
