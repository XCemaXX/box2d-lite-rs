use winit::keyboard::KeyCode;

pub enum Event {
    Restart,
    CreateBox(f32, f32),
    RunScene(usize),
}

pub struct InputState {
    pub mouse: MouseState,
    keyboard: KeyBoardState,
    events: Vec<Event>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            mouse: Default::default(),
            keyboard: Default::default(),
            events: Vec::new(),
        }
    }
}

impl InputState {
    pub fn update_mouse_buttons(&mut self, pressed: bool) {
        self.mouse.update_buttons(pressed);
        if self.mouse.left_button_pressed && self.mouse.is_cursor_inside {
            self.events.push(Event::CreateBox(self.mouse.x, self.mouse.y));
        }
    }

    pub fn update_mouse_pos(&mut self, x: f32, y: f32) {
        self.mouse.set_pos(x, y);
    }

    pub fn update_mouse_inside(&mut self, is_inside: bool) {
        self.mouse.is_cursor_inside = is_inside;
    }

    pub fn update_keyboard(&mut self, pressed: bool, key: KeyCode) {
        self.keyboard.update(pressed, key);
        if self.keyboard.space_pressed {
            self.events.push(Event::Restart);
        }
        if self.keyboard.digit_pressed {
            self.events.push(Event::RunScene(self.keyboard.digit_num));
        }
    }

    pub fn pop_event(&mut self) -> Option<Event> {
        self.events.pop()
    }
}

pub struct MouseState {
    is_cursor_inside: bool,
    left_button_pressed: bool,
    x: f32,
    y: f32,
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            is_cursor_inside: false,
            left_button_pressed: false,
            x: 0.0,
            y: 0.0,
        }
    }
}

impl MouseState {
    fn update_buttons(&mut self, pressed: bool) {
        self.left_button_pressed = pressed;
    }

    pub fn to_string(&self) -> String {
        format!("{:?}. Pos: {:.3} {:.3}", self.left_button_pressed, self.x, self.y)
    }

    fn set_pos(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }
}

#[derive(Default)]
pub struct KeyBoardState {
    space_pressed: bool,
    digit_pressed: bool,
    digit_num: usize,
}

impl KeyBoardState {
    fn update(&mut self, pressed: bool, key: KeyCode) {
        let k = match key {
            KeyCode::Space => { &mut self.space_pressed },
            KeyCode::Digit1 => { self.digit_num = 1; &mut self.digit_pressed },
            KeyCode::Digit2 => { self.digit_num = 2; &mut self.digit_pressed },
            KeyCode::Digit3 => { self.digit_num = 3; &mut self.digit_pressed },
            _ => { return; }
        };
        *k = pressed;
    }
}