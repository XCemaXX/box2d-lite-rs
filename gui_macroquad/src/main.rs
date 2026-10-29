mod buttons;
mod letterbox;

use buttons::{Action, Button};
use letterbox::Letterbox;
use physics::PhysicsState;

use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui};

// Sizes in world units; the world is [-1, 1] x [-1, 1].
const FIELD_BORDER: f32 = 0.005;
const BOX_BORDER: f32 = 0.01;
const POINT_SIZE: f32 = 0.015;
const LINE_WIDTH: f32 = 0.005;

const FPS_PERIOD: f64 = 0.4;

fn window_conf() -> Conf {
    Conf {
        window_title: "BasicShapes".to_owned(),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(miniquad::date::now() as u64);
    let mut physics_state = PhysicsState::with_jitter(|| rand::gen_range(-1.0, 1.0));
    let mut last_update = get_time();
    let font = load_ttf_font_from_bytes(include_bytes!("../../fonts/DejaVuSans.ttf")).unwrap();
    let mut letterbox = Letterbox::new(Rect::new(-1.0, -1.0, 2.0, 2.0), true);
    let buttons = buttons::create_buttons();
    let mut fps = FpsCounter::default();

    loop {
        let lu = get_time();
        let dt = lu - last_update;
        last_update = lu;
        fps.add_frame(dt);
        letterbox.update();
        letterbox.clear(GRAY_BARS, GRAY_BACKGROUND);

        root_ui().window(hash!(), Vec2::new(10., 10.), Vec2::new(280., 120.), |ui| {
            ui.label(None, &format!("fps: {:.1}", fps.value));
            ui.label(None, &format!("{}", physics_state));
            ui.label(None, "Controls: 1-9 scenes; Space: restart");
            ui.label(None, "          P, N - prev, next scene");
            ui.label(None, "          Click - add box");
        });
        let f = letterbox.field();
        let border = letterbox.len_to_screen(FIELD_BORDER);
        draw_rectangle_lines(f.x, f.y, f.w, f.h, border, BLACK);

        for b in &buttons {
            b.draw(&letterbox, &font);
        }
        update_physics_by_input(&mut physics_state, &buttons, &letterbox);

        physics_state.step(dt as f32);
        draw_physics(&physics_state, &letterbox);

        next_frame().await;
    }
}

/// Average fps, updated once per `FPS_PERIOD` so the number is readable.
#[derive(Default)]
struct FpsCounter {
    value: f64,
    frames: u32,
    time: f64,
}

impl FpsCounter {
    fn add_frame(&mut self, dt: f64) {
        self.frames += 1;
        self.time += dt;
        if self.time >= FPS_PERIOD {
            self.value = self.frames as f64 / self.time;
            self.frames = 0;
            self.time = 0.0;
        }
    }
}

fn draw_physics(physics: &PhysicsState, letterbox: &Letterbox) {
    let box_border = letterbox.len_to_screen(BOX_BORDER);
    for r in physics.rectangles() {
        let c = letterbox.to_screen(vec2(r.center.x, r.center.y));
        let (w, h) = (
            letterbox.len_to_screen(r.width),
            letterbox.len_to_screen(r.height),
        );
        // World y is up, screen y is down: the angle changes sign.
        let p = DrawRectangleParams {
            color: DARK_GREEN,
            offset: Vec2 { x: 0.5, y: 0.5 },
            rotation: -r.rotation,
        };
        draw_rectangle_ex(c.x, c.y, w, h, p); //outer
        let p = DrawRectangleParams {
            color: GRAY_BACKGROUND,
            offset: Vec2 { x: 0.5, y: 0.5 },
            rotation: -r.rotation,
        };
        draw_rectangle_ex(c.x, c.y, w - box_border, h - box_border, p); //inner
    }
    let point_size = letterbox.len_to_screen(POINT_SIZE);
    for p in physics.collide_points() {
        let c = letterbox.to_screen(vec2(p.x, p.y));
        let params = DrawRectangleParams {
            color: RED,
            offset: Vec2 { x: 0.5, y: 0.5 },
            rotation: 0.0,
        };
        draw_rectangle_ex(c.x, c.y, point_size, point_size, params);
    }
    let line_width = letterbox.len_to_screen(LINE_WIDTH);
    for l in physics.joint_lines() {
        let a = letterbox.to_screen(vec2(l.p1.x, l.p1.y));
        let b = letterbox.to_screen(vec2(l.p2.x, l.p2.y));
        draw_line(a.x, a.y, b.x, b.y, line_width, BLACK);
    }
}

fn update_physics_by_input(
    physics_state: &mut PhysicsState,
    buttons: &[Button],
    letterbox: &Letterbox,
) {
    const DIGIT_START: usize = KeyCode::Key1 as usize;
    const DIGIT_END: usize = KeyCode::Key9 as usize;
    if let Some(key) = get_last_key_pressed() {
        match key {
            KeyCode::Space => {
                physics_state.restart();
            }
            KeyCode::N => {
                physics_state.change_to_next_scene();
            }
            KeyCode::P => {
                physics_state.change_to_prev_scene();
            }
            digit => {
                if let DIGIT_START..=DIGIT_END = digit as usize {
                    let scene = digit as usize - DIGIT_START;
                    // A digit key without a scene does nothing.
                    let _ = physics_state.set_scene(scene);
                }
            }
        }
    };

    if !is_mouse_button_pressed(MouseButton::Left) {
        return;
    }
    let Some(p) = letterbox.mouse_world() else {
        return;
    };
    match buttons.iter().find(|b| b.contains(p)).map(|b| b.action) {
        Some(Action::Prev) => physics_state.change_to_prev_scene(),
        Some(Action::Restart) => physics_state.restart(),
        Some(Action::Next) => physics_state.change_to_next_scene(),
        None => physics_state.add_rectangle(p.x, p.y),
    }
}

const GRAY_BACKGROUND: Color = Color {
    r: 0.66, // 167/256
    g: 0.66,
    b: 0.66,
    a: 1.0,
};

const GRAY_BARS: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};

const DARK_GREEN: Color = Color {
    r: 0.0,
    g: 0.5,
    b: 0.0,
    a: 1.0,
};
