use crate::game::Event;
use crate::letterbox::Rect;
use physics::primitives::{Point, Triangle};

/// The world rectangle: the physics coordinates on screen.
pub const WORLD: Rect = Rect::new(-1.0, -1.0, 2.0, 2.0);

/// A button on the bottom panel of the world.
pub struct Button {
    pub rect: Rect,
    pub icon: Triangle,
    pub event: Event,
}

impl Button {
    /// `icon` points are relative to the button center.
    const fn new(cx: f32, icon: [[f32; 2]; 3], event: Event) -> Self {
        Button {
            rect: Rect::new(
                cx - BUTTON_WIDTH / 2.0,
                WORLD.y,
                BUTTON_WIDTH,
                BUTTON_HEIGHT,
            ),
            icon: Triangle {
                p1: offset(cx, icon[0]),
                p2: offset(cx, icon[1]),
                p3: offset(cx, icon[2]),
            },
            event,
        }
    }
}

const fn offset(cx: f32, d: [f32; 2]) -> Point {
    Point {
        x: cx + d[0],
        y: BUTTON_Y + d[1],
    }
}

const BUTTON_WIDTH: f32 = WORLD.w / 3.0;
const BUTTON_HEIGHT: f32 = 0.15;
const BUTTON_Y: f32 = WORLD.y + BUTTON_HEIGHT / 2.0;
const CENTER_X: f32 = WORLD.x + WORLD.w / 2.0;
// Half size of an icon.
const I: f32 = 0.03;

pub static BUTTONS: &[Button] = &[
    Button::new(
        CENTER_X - BUTTON_WIDTH,
        [[-I, 0.0], [I, -I], [I, I]],
        Event::PrevScene,
    ),
    Button::new(CENTER_X, [[0.0, I], [I, -I], [-I, -I]], Event::Restart),
    Button::new(
        CENTER_X + BUTTON_WIDTH,
        [[I, 0.0], [-I, -I], [-I, I]],
        Event::NextScene,
    ),
];
