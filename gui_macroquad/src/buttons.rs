use macroquad::prelude::*;
use crate::size_params::SizeParams;
use crate::coords_transformer::transform_coords;

pub struct Button {
    top_x: f32, 
    top_y: f32,
    w: f32,
    h: f32,
    pub name: String,
}

const GRAY_BUTTON: Color = Color {
    r: 0.77,
    g: 0.77,
    b: 0.77,
    a: 1.0,
};

impl Button {
    fn new(text: &str, font_scale: f32, 
        top_x: f32, top_y: f32, w: f32, h: f32, 
        font: &Font) -> Self 
    {
        draw_rectangle(top_x, top_y, w, h, GRAY_BUTTON);
        draw_rectangle_lines(top_x, top_y, w, h, 4.0, DARKPURPLE);
        let size = measure_text(text, None, 40, font_scale);
        draw_text_ex(
            text,
            top_x + w / 2.0 - size.width / 2.0, 
            top_y + size.height / 2.0 + h / 2.0,
            TextParams {
                font_size: 40,
                font_scale: font_scale,
                color: BLACK,
                font: Some(font),
                ..Default::default()
            },
        );
        Self { top_x, top_y, w, h, name: text.into() }
    }

    pub fn is_cursor_in(&self, x: f32, y: f32) -> bool {
        y > self.top_y && y < self.top_y + self.h &&
        x > self.top_x && x < self.top_x + self.w 
    }
}

pub fn create_buttons(size_params: &SizeParams, font: &Font) -> Vec<Button> {
    const BUTTONS_TOP: f32 = -0.85;
    let (win_x, win_y) = transform_coords(-1.0, BUTTONS_TOP, &size_params);
    let button_height = size_params.width + size_params.offset_y - win_y;
    let button_width = size_params.width / 3.0;
    
    vec![
        Button::new("Prev",    size_params.font_scale, win_x,                      win_y, button_width, button_height, font),
        Button::new("Restart", size_params.font_scale, win_x + button_width,       win_y, button_width, button_height, font),
        Button::new("Next",    size_params.font_scale, win_x + button_width * 2.0, win_y, button_width, button_height, font),
    ]
}
