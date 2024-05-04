

pub struct MouseState {
    pub is_cursor_inside: bool,
    pub is_left_pressed: bool,
    pub x: f32,
    pub y: f32,
}

impl MouseState {
    pub fn new() -> Self {
        MouseState {
            is_cursor_inside: false,
            is_left_pressed: false,
            x: 0.0,
            y: 0.0,
        }
    }

    pub fn to_string(&self) -> String {
        if self.is_left_pressed {
            format!("PRESSED. Pos: {:.3} {:.3}", self.x, self.y)
        } else if self.is_cursor_inside {
            format!("IN BOX. Pos: {:.3} {:.3}", self.x, self.y)
        } else {
            format!("OUT BOX")
        }
    }

    pub fn set_pos(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }
}