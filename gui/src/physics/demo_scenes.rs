
use crate::PhysicsState;
use rand::Rng;

type InitDemoScene = fn(&mut PhysicsState);
const DEMO_SCENES: &'static [(&'static str, InitDemoScene)] = &[
    ("Two boxes", init_two_boxes_scene),
    ("Simple pendulum", init_simple_pendulum),
    ("Varying friction coefficients", init_varying_friction_coefficients),
    ("Vertical stack", init_vertical_stack_scene),
    ("Pyramid", init_pyramid),
    ("teeter", init_teeter),
    ("suspension bridge", init_bridge),
    ("Dominos", init_dominos),
    ("multi-pendulum", init_multi_pendulum),
];

const FLOOR_W: f32 = 0.1;
const FLOOR_Y_CENTER: f32 = -0.8;
const FLOOR_Y: f32 = FLOOR_Y_CENTER + FLOOR_W / 2.0;
const SCREEN_WIDTH: f32 = 2.0;

fn init_two_boxes_scene(state: &mut PhysicsState) {
    let _big = state.add_body(0.2, 0.2, 200.0, 0.0, FLOOR_Y + 0.55);
    let _small = state.add_body(0.1, 0.1, 100.0, 0.1, FLOOR_Y + 0.35);
    let _floor = state.add_unmovable_body(SCREEN_WIDTH, FLOOR_W, 0.0, FLOOR_Y_CENTER);
}

fn init_simple_pendulum(state: &mut PhysicsState) {
    let center_y = 0.4;
    let r = 0.8;
    let angle = std::f32::consts::PI + std::f32::consts::PI / 4.0;
    let x = r * f32::cos(angle);
    let y = center_y + r * f32::sin(angle);

    let body1 = state.add_unmovable_body(SCREEN_WIDTH, FLOOR_W, 0.0, FLOOR_Y_CENTER);
    let body2 = state.add_body(0.2, 0.2, 500.0, x, y);

    let _anchor_ref = state.add_unmovable_body(0.05, 0.05, 0.0, center_y);
    state.add_joint(body1, body2, 0.0, center_y);
}

fn init_varying_friction_coefficients(state: &mut PhysicsState) {    
    let slide = state.add_unmovable_body(1.1, 0.04, -0.2, FLOOR_Y + 1.25);
    slide.borrow_mut().rotation = -0.25;
    let slide = state.add_unmovable_body(1.1, 0.04, 0.05, FLOOR_Y + 0.75);
    slide.borrow_mut().rotation = 0.25;
    let slide = state.add_unmovable_body(1.1, 0.04, -0.25, FLOOR_Y + 0.25);
    slide.borrow_mut().rotation = -0.25;

    let _wall = state.add_unmovable_body(0.04, 0.3, 0.55, FLOOR_Y + 1.05);
    let _wall = state.add_unmovable_body(0.04, 0.3, -0.75, FLOOR_Y + 0.55);

    let friction = [0.75, 0.5, 0.35, 0.1, 0.0];
    for i in 0..5 {
        let body = state.add_body(0.1, 0.1, 10.0, -0.65 + i as f32 * 0.15, FLOOR_Y + 1.65);
        body.borrow_mut().friction = friction[i];
    }

    let _floor = state.add_unmovable_body(SCREEN_WIDTH, FLOOR_W, 0.0, FLOOR_Y_CENTER);
}

fn init_vertical_stack_scene(state: &mut PhysicsState) {
    const W: f32 = 0.15;
    const UNSTABLE_COEF: f32 = 1.0 / 4.0;
    const STABLE_COEF: f32 = 1.0 / 6.0;
    const OFFSET_LIMIT: std::ops::Range<f32> = -W / 8.0 .. W / 8.0;
    for i in 0..6 {
        let offset = if i % 2 == 0 { -W * UNSTABLE_COEF } else { W * UNSTABLE_COEF };
        let _unstable = state.add_body(W, W, 500.0, -0.7 + offset, FLOOR_Y + W / 2.0 + i as f32 * W);

        let offset = if i % 2 == 0 { -W * STABLE_COEF } else { W * STABLE_COEF };
        let _stable = state.add_body(W, W, 500.0, 0.4 + offset, FLOOR_Y + W / 2.0 + i as f32 * W);

        let offset = rand::thread_rng().gen_range(OFFSET_LIMIT);
        let _random = state.add_body(W, W, 500.0, 0.8 + offset, FLOOR_Y + W / 2.0 + i as f32 * W);
    }
    let _floor = state.add_unmovable_body(SCREEN_WIDTH, FLOOR_W, 0.0, FLOOR_Y_CENTER);
}

fn init_pyramid(state: &mut PhysicsState) {
    const W: f32 = 0.12;
    let count = 12;
    let start = (SCREEN_WIDTH - W * count as f32) / 2.0 - SCREEN_WIDTH / 2.0;
    let extra_space = W / 3.0;
    for j in 0..count {
        for i in 0..(count-j) {
            let i = i as f32;
            let j = j as f32;
            state.add_body(W, W, 1000.0, 
                start + i * W + j * W / 2.0, 
                FLOOR_Y + W / 2.0 + j * (W + extra_space) + extra_space);
        }
    }
    
    state.add_unmovable_body(SCREEN_WIDTH, FLOOR_W, 0.0, FLOOR_Y_CENTER);
}
fn init_teeter(state: &mut PhysicsState) {
    let anchor_y = FLOOR_Y + 0.13;
    let teeter_w = 1.2;
    
    let teeter = state.add_body(teeter_w, 0.05, 200.0, 0.0, anchor_y);
    let floor = state.add_unmovable_body(SCREEN_WIDTH, FLOOR_W, 0.0, FLOOR_Y_CENTER);
    state.add_joint(teeter, floor, 0.0, anchor_y);

    let _big = state.add_body(0.2, 0.2, 500.0, teeter_w / 2.0 - 0.1, FLOOR_Y + 0.8);
    let _small1 = state.add_body(0.1, 0.1, 250.0, -(teeter_w / 2.0 - 0.04), FLOOR_Y + 0.3);
    let _small2 = state.add_body(0.1, 0.1, 250.0, -(teeter_w / 2.0 - 0.14), FLOOR_Y + 0.3);
}
fn init_bridge(state: &mut PhysicsState) {
    //todo!()
}
fn init_dominos(state: &mut PhysicsState) {
    //todo!()
}
fn init_multi_pendulum(state: &mut PhysicsState) {
    //todo!()
}

pub fn init_scene(scene: usize, state: &mut PhysicsState) {
    DEMO_SCENES[scene].1(state);
}

pub fn get_scene_name(scene: usize) -> &'static str {
    DEMO_SCENES[scene].0
}