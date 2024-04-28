

pub struct MouseState {
    pub is_cursor_inside: bool,
    pub is_left_pressed: bool,
    pub x: f64,
    pub y: f64,
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
            format!("PRESSED. Pos: {} {}", self.x.round(), self.y.round())
        } else if self.is_cursor_inside {
            format!("IN BOX. Pos: {} {}", self.x.round(), self.y.round())
        } else {
            format!("OUT BOX")
        }
    }
}