use std::rc::Rc;
use std::cell::RefCell;

use super::box2d::{World, Body, Vec2};
use super::primitives::{Rectangle, Point};

use super::box2d::{collide, Contact, MAX_CONTACT_POINT};

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
        square.position.set(0.0, 0.3);
        bodies_storage.push(Rc::new(RefCell::new(square)));
        
        let mut square = Body::default();
        square.set(Vec2::new(2.0, 0.2), f32::MAX);
        square.position.set(0.0, -0.5);
        //square.set(Vec2::new(0.3, 0.3), f32::MAX);
        //square.position.set(-0.0, -0.5);
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

    pub fn get_rectangles(&self) -> Vec<Rectangle> {
        let mut res = Vec::new();
        for body in &self.bodies_storage {
            let body = body.borrow();
            res.push(Rectangle {
                x: body.position.x - body.width.x / 2.0,
                y: body.position.y + body.width.y / 2.0,
                h: body.width.y,
                w: body.width.x,
            });
        }
        res
    }

    pub fn get_collide_points(&self) -> Vec<Point> {
        let body_a = &self.bodies_storage[0].borrow();
        let body_b = &self.bodies_storage[1].borrow();

        let mut contacts: [Contact; MAX_CONTACT_POINT] = Default::default();
        let count = collide(&mut contacts, body_a, body_b);
        let mut res = Vec::new();
        for i in 0..count {
            res.push(Point{
                x: contacts[i].position.x, 
                y: contacts[i].position.y,
                });
        }
        res
    }
}

