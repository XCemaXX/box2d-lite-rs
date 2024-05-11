use std::rc::Rc;
use std::cell::RefCell;

use super::box2d::{World, Body, Vec2, UNMOVABLE_MASS};
use super::primitives::{Rectangle, Point};

pub struct WorldState {
    world: World,
    pub bodies_storage: Vec<Rc<RefCell<Body>>>,
    pub current_scene: usize,
}

mod demo_scenes {
    use crate::WorldState;
    type InitDemoScene = fn(&mut WorldState);
    const DEMO_SCENES: &'static [(&'static str, InitDemoScene)] = &[
        ("dummy0", dummy_scene),
        ("Two boxes", init_two_boxes_scene),
        ("Vertical stack", init_vertical_stack_scene),
    ];

    fn dummy_scene(_: &mut WorldState) { }

    fn init_two_boxes_scene(world_state: &mut WorldState) {
        world_state.add_body(0.2, 0.2, 200.0, 0.0, 0.0);
        world_state.add_body(0.1, 0.1, 100.0, 0.1, -0.2);
        world_state.add_unmovable_body(2.0, 0.2, 0.0, -0.5);
    }

    fn init_vertical_stack_scene(world_state: &mut WorldState) {
        let w = 0.15;
        for i in 0..6 {
            let offset = if i % 2 == 0 { -w / 4.0 } else { w / 4.0 };
            world_state.add_body(w, w, 500.0, -0.5 + offset, -0.7 + w / 2.0 + i as f32 * w);

            let offset = if i % 2 == 0 { -w / 5.0 } else { w / 5.0 };
            world_state.add_body(w, w, 500.0, 0.5 + offset, -0.7 + w / 2.0 + i as f32 * w);
        }
        world_state.add_unmovable_body(2.0, 0.2, 0.0, -0.8);
    }

    pub fn run_demo(scene: usize, world_state: &mut WorldState) {
        DEMO_SCENES[scene].1(world_state);
    }

    pub fn get_scene_name(scene: usize) -> &'static str {
        DEMO_SCENES[scene].0
    }
}

const SCALE_MULT: f32 = 100.0;
const GRAVITY: f32 = -20.0;

impl WorldState {
    fn add_unmovable_body(&mut self, w: f32, h: f32, pos_x: f32, pos_y: f32) {
        self.add_body(w, h, UNMOVABLE_MASS, pos_x, pos_y);
    }

    fn add_body(&mut self, w: f32, h: f32, mass: f32, pos_x: f32, pos_y: f32) {
        let body = Rc::new(RefCell::new(
            Body::new(w * SCALE_MULT, h * SCALE_MULT, mass, pos_x * SCALE_MULT, pos_y * SCALE_MULT)
        ));
        self.bodies_storage.push(body.clone());
        self.world.add_body(body);
    }

    pub fn new(demo_scene: usize) -> Self {
        let gravity = Vec2::new(0.0, GRAVITY);
        let iterations = 10;
        let mut world_state = WorldState{ 
            world: World::new(gravity, iterations), 
            bodies_storage: Vec::new(),
            current_scene: demo_scene,
        };
        demo_scenes::run_demo(demo_scene, &mut world_state);
        world_state
    }

    pub fn step(&mut self, dt: f32) {
        self.world.step(dt);
    }

    pub fn get_rectangles(&self) -> Vec<Rectangle> {
        let mut res = Vec::new();
        for body in &self.bodies_storage {
            let body = body.borrow();
            let center = &body.position;
        
            res.push(Rectangle {
                center: Point{ x: center.x / SCALE_MULT, y: center.y / SCALE_MULT },
                width: body.width.x / SCALE_MULT,
                height: body.width.y / SCALE_MULT,
                rotation: body.rotation,
            });
        }
        res
    }

    pub fn get_collide_points(&self) -> Vec<Point> {
        let mut res = Vec::new();
        for p in self.world.get_collide_points() {
            res.push(Point {
                x: p.x / SCALE_MULT,
                y: p.y / SCALE_MULT,
            })
        }
        res
    }

    pub fn add_rectangle(&mut self, x: f32, y: f32) {
        self.add_body(0.15, 0.15, 500.0, x, y);
    }

    pub fn get_scene_name(&self) -> String {
        format!("Scene {}: {}", self.current_scene, demo_scenes::get_scene_name(self.current_scene))
    }
}

