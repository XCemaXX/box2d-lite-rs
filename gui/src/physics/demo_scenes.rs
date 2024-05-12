
use crate::PhysicsState;

type InitDemoScene = fn(&mut PhysicsState);
const DEMO_SCENES: &'static [(&'static str, InitDemoScene)] = &[
    ("Two boxes", init_two_boxes_scene),
    ("Vertical stack", init_vertical_stack_scene),
    ("Simple pendulum", init_simple_pendulum),
];

fn init_two_boxes_scene(state: &mut PhysicsState) {
    state.add_body(0.2, 0.2, 200.0, 0.0, 0.0);
    state.add_body(0.1, 0.1, 100.0, 0.1, -0.2);
    state.add_unmovable_body(2.0, 0.2, 0.0, -0.5);
}

fn init_vertical_stack_scene(state: &mut PhysicsState) {
    let w = 0.15;
    for i in 0..6 {
        let offset = if i % 2 == 0 { -w / 4.0 } else { w / 4.0 };
        state.add_body(w, w, 500.0, -0.5 + offset, -0.7 + w / 2.0 + i as f32 * w);

        let offset = if i % 2 == 0 { -w / 5.0 } else { w / 5.0 };
        state.add_body(w, w, 500.0, 0.5 + offset, -0.7 + w / 2.0 + i as f32 * w);
    }
    state.add_unmovable_body(2.0, 0.2, 0.0, -0.8);
}

fn init_simple_pendulum(state: &mut PhysicsState) {
    let center_y = 0.4;
    let r = 0.8;
    let angle = std::f32::consts::PI + std::f32::consts::PI / 4.0;
    let x = r * f32::cos(angle);
    let y = center_y + r * f32::sin(angle);

    let body1 = state.add_unmovable_body(2.0, 0.2, 0.0, -0.8);
    let body2 = state.add_body(0.2, 0.2, 500.0, x, y);

    let _anchor_ref = state.add_unmovable_body(0.05, 0.05, 0.0, center_y);
    state.add_joint(body1, body2, 0.0, center_y);
}

pub fn init_scene(scene: usize, state: &mut PhysicsState) {
    DEMO_SCENES[scene].1(state);
}

pub fn get_scene_name(scene: usize) -> &'static str {
    DEMO_SCENES[scene].0
}