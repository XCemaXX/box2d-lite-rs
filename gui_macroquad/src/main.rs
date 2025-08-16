mod buttons;
mod coords_transformer;
mod size_params;

use buttons::Button;
use coords_transformer::*;
use physics::PhysicsState;
use physics::primitives::{Line, Point, Rectangle};
use size_params::SizeParams;

use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui};

fn window_conf() -> Conf {
    Conf {
        window_title: "BasicShapes".to_owned(),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut physics_state = PhysicsState::new(0);
    let mut last_update = get_time();
    let font = load_ttf_font_from_bytes(include_bytes!("../../fonts/DejaVuSans.ttf")).unwrap();

    loop {
        clear_background(GRAY_BACKGROUND);
        let lu = get_time();
        let dt = lu - last_update;
        last_update = lu;
        let size_params = SizeParams::new();

        root_ui().window(hash!(), Vec2::new(10., 10.), Vec2::new(280., 120.), |ui| {
            ui.label(None, &format!("fps: {:.3}", 1.0 / dt));
            ui.label(None, &format!("{}", physics_state));
            ui.label(None, &format!("Controls: 1-9 scenes; Space: restart"));
            ui.label(None, &format!("          P, N - prev, next scene"));
            ui.label(None, &format!("          Click - add box"));
        });
        draw_rectangle_lines(
            size_params.offset_x,
            size_params.offset_y,
            size_params.width,
            size_params.width,
            2.0,
            BLACK,
        );

        let buttons = buttons::create_buttons(&size_params, &font);
        update_physics_by_input(&mut physics_state, buttons, &size_params);

        physics_state.step(dt as f32);
        let rectangles = physics_state.get_rectangles();
        let collide_points = physics_state.get_collide_points();
        let joint_lines = physics_state.get_joint_lines();
        update_frame_physics(rectangles, collide_points, joint_lines, &size_params);

        next_frame().await;
    }
}

fn update_frame_physics(
    rectangles: Vec<Rectangle>,
    points: Vec<Point>,
    lines: Vec<Line>,
    size_params: &SizeParams,
) {
    for r in &rectangles {
        let (x, y, w, h) = rectangle_transform(r, &size_params);
        let p = DrawRectangleParams {
            color: DARK_GREEN,
            offset: Vec2 { x: 0.5, y: 0.5 },
            rotation: -r.rotation,
        };
        draw_rectangle_ex(x, y, w, h, p); //outer
        let p = DrawRectangleParams {
            color: GRAY_BACKGROUND,
            offset: Vec2 { x: 0.5, y: 0.5 },
            rotation: -r.rotation,
        };
        let frame_width = 4.0;
        draw_rectangle_ex(x, y, w - frame_width, h - frame_width, p); //inner
    }
    for c in &points {
        let (x, y) = transform_coords(c.x, c.y, &size_params);
        let p = DrawRectangleParams {
            color: RED,
            offset: Vec2 { x: 0.5, y: 0.5 },
            rotation: 0.0,
        };
        draw_rectangle_ex(x, y, 6.0, 6.0, p);
    }
    for l in &lines {
        let (x1, y1) = transform_coords(l.p1.x, l.p1.y, &size_params);
        let (x2, y2) = transform_coords(l.p2.x, l.p2.y, &size_params);
        draw_line(x1, y1, x2, y2, 2.0, BLACK);
    }
}

fn update_physics_by_input(
    physics_state: &mut PhysicsState,
    buttons: Vec<Button>,
    size_params: &SizeParams,
) {
    let (mouse_x, mouse_y) = mouse_position();

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
            digit => match digit as usize {
                DIGIT_START..=DIGIT_END => {
                    let scene = digit as usize - DIGIT_START;
                    *physics_state = PhysicsState::new(scene);
                }
                _ => {}
            },
        }
    };
    if mouse_x > size_params.offset_x
        && mouse_x < size_params.offset_x + size_params.width
        && mouse_y > size_params.offset_y
        && mouse_y < size_params.offset_y + size_params.width
        && is_mouse_button_pressed(MouseButton::Left)
    {
        let pressed = 'pressed: {
            for b in &buttons {
                if b.is_cursor_in(mouse_x, mouse_y) {
                    match b.name.as_str() {
                        "Next" => {
                            physics_state.change_to_next_scene();
                        }
                        "Restart" => {
                            physics_state.restart();
                        }
                        "Prev" => {
                            physics_state.change_to_prev_scene();
                        }
                        _ => {}
                    };
                    break 'pressed true;
                }
            }
            false
        };
        if !pressed {
            let (x, y) = transform_coords_back(mouse_x, mouse_y, &size_params);
            physics_state.add_rectangle(x, y);
        }
    }
}

const GRAY_BACKGROUND: Color = Color {
    r: 0.66, // 167/256
    g: 0.66,
    b: 0.66,
    a: 1.0,
};

const DARK_GREEN: Color = Color {
    r: 0.0,
    g: 0.5,
    b: 0.0,
    a: 1.0,
};
