use crate::letterbox::Letterbox;
use macroquad::prelude::*;

#[derive(Clone, Copy)]
pub enum Action {
    Prev,
    Restart,
    Next,
}

pub struct Button {
    rect: Rect,
    label: &'static str,
    pub action: Action,
}

const GRAY_BUTTON: Color = Color {
    r: 0.77,
    g: 0.77,
    b: 0.77,
    a: 1.0,
};

// Rasterized once; on usual screens text is scaled down, so it stays sharp.
const FONT_SIZE: u16 = 64;
const TEXT_SIZE: f32 = 0.1;
const BORDER: f32 = 0.01;

impl Button {
    pub fn contains(&self, p: Vec2) -> bool {
        self.rect.contains(p)
    }

    pub fn draw(&self, letterbox: &Letterbox, font: &Font) {
        let r = letterbox.rect_to_screen(self.rect);
        draw_rectangle(r.x, r.y, r.w, r.h, GRAY_BUTTON);
        let border = letterbox.len_to_screen(BORDER);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, border, DARKPURPLE);

        let font_scale = letterbox.font_scale(FONT_SIZE, TEXT_SIZE);
        let size = measure_text(self.label, Some(font), FONT_SIZE, font_scale);
        draw_text_ex(
            self.label,
            r.x + (r.w - size.width) / 2.0,
            r.y + (r.h + size.height) / 2.0,
            TextParams {
                font_size: FONT_SIZE,
                font_scale,
                color: BLACK,
                font: Some(font),
                ..Default::default()
            },
        );
    }
}

/// A row of buttons at the bottom of the world `[-1, 1] x [-1, 1]`.
pub fn create_buttons() -> [Button; 3] {
    const BOTTOM: f32 = -1.0;
    const HEIGHT: f32 = 0.15;
    const WIDTH: f32 = 2.0 / 3.0;
    let button = |i: f32, label, action| Button {
        rect: Rect::new(-1.0 + WIDTH * i, BOTTOM, WIDTH, HEIGHT),
        label,
        action,
    };
    [
        button(0.0, "Prev", Action::Prev),
        button(1.0, "Restart", Action::Restart),
        button(2.0, "Next", Action::Next),
    ]
}
