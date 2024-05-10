use std::rc::Rc;
use std::cell::RefCell;

use super::box2d::{World, Body, Vec2, UNMOVABLE_MASS};
use super::primitives::{Rectangle, Point};

pub struct WorldState {
    world: World,
    pub bodies_storage: Vec<Rc<RefCell<Body>>>,
}

const SCALE_MULT: f32 = 100.0;
const GRAVITY: f32 = -20.0;

fn new_scaled_body(w: f32, h: f32, mass: f32, pos_x: f32, pos_y: f32) -> Rc<RefCell<Body>> {
    Rc::new(RefCell::new(
        Body::new(w * SCALE_MULT, h * SCALE_MULT, mass, pos_x * SCALE_MULT, pos_y * SCALE_MULT)
    ))
}

impl WorldState {
    fn make_bodies() -> Vec<Rc<RefCell<Body>>>{
        let mut bodies_storage = Vec::new();
        bodies_storage.push(new_scaled_body(0.2, 0.2, 200.0, 0.0, 0.0));

        bodies_storage.push(new_scaled_body(0.1, 0.1, 100.0, 0.1, -0.2));

        bodies_storage.push(new_scaled_body(2.0, 0.2, UNMOVABLE_MASS, 0.0, -0.5));
        bodies_storage
    }

    pub fn new() -> Self {
        let gravity = Vec2::new(0.0, GRAVITY);
        let iterations = 10;
        
        let mut world_state = WorldState{ 
            world: World::new(gravity, iterations), 
            bodies_storage: Self::make_bodies() 
        };
        for body in &world_state.bodies_storage {
            world_state.world.add_body(body.clone());
        }
        world_state
    }

    pub fn step(&mut self, dt: f32) {
        self.world.step(dt);
    }

    pub fn get_rectangles(&self) -> Vec<Rectangle> {
        let mut res = Vec::new();
        for body in &self.bodies_storage {
            let body = body.borrow();
            res.push(Rectangle {
                x: (body.position.x - body.width.x / 2.0) / SCALE_MULT,
                y: (body.position.y + body.width.y / 2.0) / SCALE_MULT,
                h: body.width.y / SCALE_MULT,
                w: body.width.x / SCALE_MULT,
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
}

