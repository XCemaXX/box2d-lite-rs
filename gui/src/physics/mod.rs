mod demo_scenes;

use std::rc::Rc;
use std::cell::RefCell;

use box2d::{World, Body, Joint, Vec2, UNMOVABLE_MASS};
use crate::primitives::{Rectangle, Point, Line};

pub struct PhysicsState {
    world: World,
    current_scene: usize,
}

const SCALE_MULT: f32 = 100.0;
const GRAVITY: f32 = -500.0;

impl PhysicsState {
    fn add_unmovable_body(&mut self, w: f32, h: f32, pos_x: f32, pos_y: f32) -> Rc<RefCell<Body>> {
        self.add_body(w, h, UNMOVABLE_MASS, pos_x, pos_y)
    }

    fn add_body(&mut self, w: f32, h: f32, mass: f32, pos_x: f32, pos_y: f32) -> Rc<RefCell<Body>> {
        let body = Rc::new(RefCell::new(
            Body::new(w * SCALE_MULT, h * SCALE_MULT, mass, pos_x * SCALE_MULT, pos_y * SCALE_MULT)
        ));
        self.world.add_body(body.clone());
        body
    }

    fn add_joint(&mut self, body1: Rc<RefCell<Body>>, body2: Rc<RefCell<Body>>, anchor_x: f32, anchor_y: f32) -> Rc<RefCell<Joint>> {
        let joint = self.world.add_joint(body1, body2, &Vec2::new(anchor_x * SCALE_MULT, anchor_y * SCALE_MULT));
        joint
    }

    pub fn new(demo_scene: usize) -> Self {
        let gravity = Vec2::new(0.0, GRAVITY);
        let iterations = 10;
        let mut state = PhysicsState{ 
            world: World::new(gravity, iterations),
            current_scene: demo_scene,
        };
        demo_scenes::init_scene(demo_scene, &mut state);
        state
    }

    pub fn step(&mut self, dt: f32) {
        self.world.step(dt);
    }

    pub fn get_rectangles(&self) -> Vec<Rectangle> {
        self.world.get_bodies().iter().map(|body| {
            let body = body.borrow();
            let center = &body.position;
            Rectangle {
                center: Point{ x: center.x / SCALE_MULT, y: center.y / SCALE_MULT },
                width: body.width.x / SCALE_MULT,
                height: body.width.y / SCALE_MULT,
                rotation: body.rotation,
            }
        }).collect()
    }

    pub fn get_collide_points(&self) -> Vec<Point> {
        self.world.get_collide_points().iter()
            .map(|p| { Point { 
                x: p.x / SCALE_MULT,
                y: p.y / SCALE_MULT,
            }}).collect()
    }

    pub fn get_joint_lines(&self) -> Vec<Line> {
        self.world.get_joint_lines().iter()
            .map(|(start, end)| { Line {
                p1: Point { x: start.x / SCALE_MULT, y: start.y / SCALE_MULT },
                p2: Point { x: end.x / SCALE_MULT,   y: end.y / SCALE_MULT },
            }}).collect()
    }

    pub fn add_rectangle(&mut self, x: f32, y: f32) {
        self.add_body(0.15, 0.15, 200.0, x, y);
    }

    pub fn restart(&mut self) {
        let scene = self.current_scene;
        *self = PhysicsState::new(scene); 
    }

    pub fn change_to_next_scene(&mut self) {
        let scene = (self.current_scene + 1) % demo_scenes::get_scene_amount();
        *self = PhysicsState::new(scene);   
    }

    pub fn change_to_prev_scene(&mut self) {
        let scene = if self.current_scene == 0 { demo_scenes::get_scene_amount() } else { self.current_scene };
        *self = PhysicsState::new(scene - 1);   
    }
}

impl std::fmt::Display for PhysicsState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Scene {}: {}. Bodies: {}", 
            self.current_scene + 1, 
            demo_scenes::get_scene_name(self.current_scene), 
            self.world.bodies_size())
    }
}