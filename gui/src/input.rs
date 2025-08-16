use crate::buttons::BUTTONS;
use winit::keyboard::KeyCode;

pub enum Event {
    Restart,
    CreateBox(f32, f32),
    RunScene(usize),
    ChangeToNextScene,
    ChangeToPrevScene,
}

pub struct InputState {
    cursor: CursorState,
    keyboard: KeyBoardState,
    events: Vec<Event>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            cursor: Default::default(),
            keyboard: Default::default(),
            events: Vec::new(),
        }
    }
}

impl InputState {
    pub fn update_cursor_buttons(&mut self, pressed: bool) {
        self.cursor.update_buttons(pressed);
        if self.cursor.button_pressed {
            for (name, b) in BUTTONS {
                if b.in_button(self.cursor.x, self.cursor.y) {
                    match name {
                        &"Prev" => {
                            self.events.push(Event::ChangeToPrevScene);
                        }
                        &"Restart" => {
                            self.events.push(Event::Restart);
                        }
                        &"Next" => {
                            self.events.push(Event::ChangeToNextScene);
                        }
                        _ => {}
                    }
                    return;
                }
            }
            self.events
                .push(Event::CreateBox(self.cursor.x, self.cursor.y));
        }
    }

    pub fn update_cursor_pos(&mut self, x: f32, y: f32) {
        self.cursor.set_pos(x, y);
    }

    pub fn update_keyboard(&mut self, pressed: bool, key: KeyCode) {
        self.keyboard.update(pressed, key);
        if self.keyboard.space_pressed {
            self.events.push(Event::Restart);
        }
        if self.keyboard.digit_pressed {
            self.events.push(Event::RunScene(self.keyboard.digit_num));
        }
        if self.keyboard.letter_pressed {
            match self.keyboard.letter {
                KeyLetters::N => self.events.push(Event::ChangeToNextScene),
                KeyLetters::P => self.events.push(Event::ChangeToPrevScene),
                _ => {}
            }
        }
    }

    pub fn pop_event(&mut self) -> Option<Event> {
        self.events.pop()
    }
}

impl std::fmt::Display for InputState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mouse: {:.3} {:.3}", self.cursor.x, self.cursor.y)
    }
}

pub struct CursorState {
    button_pressed: bool,
    x: f32,
    y: f32,
}

impl Default for CursorState {
    fn default() -> Self {
        Self {
            button_pressed: false,
            x: 0.0,
            y: 0.0,
        }
    }
}

impl CursorState {
    fn update_buttons(&mut self, pressed: bool) {
        self.button_pressed = pressed;
    }

    fn set_pos(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }
}

#[derive(Debug)]
enum KeyLetters {
    None,
    N,
    P,
}

impl Default for KeyLetters {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Default)]
pub struct KeyBoardState {
    space_pressed: bool,
    digit_pressed: bool,
    digit_num: usize,
    letter_pressed: bool,
    letter: KeyLetters,
}

impl KeyBoardState {
    fn update(&mut self, pressed: bool, key: KeyCode) {
        const DIGIT_START: usize = KeyCode::Digit1 as usize;
        const DIGIT_END: usize = KeyCode::Digit9 as usize;

        let k = match key {
            KeyCode::Space => &mut self.space_pressed,
            KeyCode::KeyN => {
                self.letter = KeyLetters::N;
                &mut self.letter_pressed
            }
            KeyCode::KeyP => {
                self.letter = KeyLetters::P;
                &mut self.letter_pressed
            }
            digit => match digit as usize {
                DIGIT_START..=DIGIT_END => {
                    self.digit_num = digit as usize - DIGIT_START;
                    &mut self.digit_pressed
                }
                _ => {
                    return;
                }
            },
        };
        *k = pressed;
    }
}
