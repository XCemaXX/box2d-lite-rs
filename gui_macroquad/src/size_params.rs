
use macroquad::prelude::{screen_height, screen_width};

pub struct SizeParams {
    pub width: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub font_scale: f32, 
}

impl SizeParams {
    pub fn new() -> Self {
        let width = screen_width().min(screen_height());
        let offset_x = (screen_width() - width) / 2.;
        let offset_y = (screen_height() - width) / 2.;
        Self { width, offset_x, offset_y,
            font_scale: width / 800.0 }
    }
}