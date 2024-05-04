use std::rc::Rc;
use std::cell::RefCell;

use super::box2d::{World, Body, Vec2};
use super::rectangle;

pub struct WorldState {
    world: World,
    pub bodies_storage: Vec<Rc<RefCell<Body>>>,
}

impl WorldState {
    fn make_bodies() -> Vec<Rc<RefCell<Body>>>{
        let mut bodies_storage = Vec::new();
        let mut square = Body::default();
        //square.set(Vec2::new(1.0, 1.0), 200.0);
        //square.position.set(0.0, 4.0);
        square.set(Vec2::new(0.2, 0.2), 200.0);
        square.position.set(-0.1, 0.3);
        bodies_storage.push(Rc::new(RefCell::new(square)));
        
        let mut square = Body::default();
        square.set(Vec2::new(2.0, 0.2), f32::MAX);
        square.position.set(-1.0, -0.5);
        bodies_storage.push(Rc::new(RefCell::new(square)));
        //b->Set(Vec2(100.0f, 20.0f), FLT_MAX);
	    //b->position.Set(0.0f, -0.5f * b->width.y);
        bodies_storage
    }

    pub fn new() -> Self {
        //let gravity = Vec2::new(0.0, -10.0);
        let gravity = Vec2::new(0.0, -0.04);
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

    pub fn get_rectangles(&self) -> Vec<rectangle::Rectangle> {
        let mut res = Vec::new();
        for body in &self.bodies_storage {
            let body = body.borrow();
            res.push(rectangle::Rectangle {
                x: body.position.x,
                y: body.position.y,
                h: body.width.y,
                w: body.width.x,
            });
        }
        res
    }
}

