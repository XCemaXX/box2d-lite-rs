use crate::primitives::{Point, Rectangle, Triangle};
pub struct Button {
    pub rect: Rectangle,
    pub icon: Triangle,
}

impl Button {
    const fn new(center: Point, w: f32, h: f32, icon: Triangle) -> Self {
        Button { rect: Rectangle { center, width: w, height: h, rotation: 0.0}, icon }
    }

    const fn new_on_bot_panel(x: f32, icon: Triangle) -> Self {
        Button::new(Point { x, y: BUTTON_Y_POS }, BUTTON_WIDTH, BUTTON_HEIGHT, icon)
    }

    pub fn in_button(&self, x: f32, y: f32) -> bool {
        let r = &self.rect;
        r.center.y + r.height / 2.0 > y 
        && r.center.y - r.height / 2.0 < y
        && r.center.x - r.width / 2.0 < x
        && r.center.x + r.width / 2.0 > x 
    }
}

pub static BUTTONS: &'static [(&'static str, Button)] = &[
    ("Prev",    Button::new_on_bot_panel(-BUTTON_WIDTH, LEFT_TRIANGLE)),
    ("Restart", Button::new_on_bot_panel(0.0,           TOP_TRIANGLE)),
    ("Next",    Button::new_on_bot_panel(BUTTON_WIDTH,  RIGHT_TRIANGLE)),
];

const SCREEN_WH:      f32 = 2.0;
const BUTTON_WIDTH:   f32 = SCREEN_WH / 3.0;
const BUTTON_HEIGHT:  f32 = 0.15;
const BUTTON_Y_POS:   f32 = -SCREEN_WH / 2.0 + BUTTON_HEIGHT / 2.0;
const ICON_WIDTH:     f32 = 0.03;

const LEFT_TRIANGLE: Triangle = Triangle {
    p1: Point{ x: -BUTTON_WIDTH - ICON_WIDTH, y: BUTTON_Y_POS },
    p2: Point{ x: -BUTTON_WIDTH + ICON_WIDTH, y: BUTTON_Y_POS - ICON_WIDTH },
    p3: Point{ x: -BUTTON_WIDTH + ICON_WIDTH, y: BUTTON_Y_POS + ICON_WIDTH },
};

const RIGHT_TRIANGLE: Triangle = Triangle {
    p1: Point{ x: BUTTON_WIDTH + ICON_WIDTH, y: BUTTON_Y_POS },
    p2: Point{ x: BUTTON_WIDTH - ICON_WIDTH, y: BUTTON_Y_POS - ICON_WIDTH },
    p3: Point{ x: BUTTON_WIDTH - ICON_WIDTH, y: BUTTON_Y_POS + ICON_WIDTH },
};

const TOP_TRIANGLE: Triangle = Triangle {
    p1: Point{ x: 0.0, y: BUTTON_Y_POS + ICON_WIDTH },
    p2: Point{ x: ICON_WIDTH, y: BUTTON_Y_POS - ICON_WIDTH },
    p3: Point{ x: -ICON_WIDTH, y: BUTTON_Y_POS - ICON_WIDTH },
};