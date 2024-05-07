use super::math_utils::{Vec2, Mat22};
use super::body::Body;

use std::rc::Rc;
use std::cell::RefCell;

pub struct World {
    gravity: Vec2,
    iterations: i32,
    bodies: Vec<Rc<RefCell<Body>>>,
}

impl World {
    pub fn new(gravity: Vec2, iterations: i32) -> Self {
        Self {
            gravity,
            iterations,
            bodies: Vec::new(),
        }
    }

    pub fn add_body(&mut self, body: Rc<RefCell<Body>>) {
        self.bodies.push(body);
    }

    pub fn clear(&mut self) {
        self.bodies.clear();
    }

    pub fn step(&mut self, dt: f32) {
        let inv_dt = if dt > 0.0 { 1.0 / dt } else { 0.0 };

        // Determine overlapping bodies and update contact points.
	    self.broad_phase();

        // Integrate forces.
        for body in &mut self.bodies {
            let inv_mass = body.borrow().inv_mass;
            let force = body.borrow().force;
            let mut body = body.borrow_mut();

            if body.inv_mass == 0.0 {
                continue;
            }
            body.velocity += &(dt * &(&self.gravity + &(inv_mass * &force)));
            body.angular_velocity += dt * body.inv_i * body.torque;
        }
        // Perform pre-steps.
        // TODO
        // Perform iterations
        // TODO
        
        // Integrate Velocities
        for body in &mut self.bodies {
            let velocity = body.borrow().velocity;
            let angular_velocity = body.borrow().angular_velocity;
            let mut body = body.borrow_mut();

            body.position += &(dt * &velocity);
            body.rotation += dt * angular_velocity;
            body.force.set(0.0, 0.0);
            body.torque = 0.0;
        } 
    }

    pub fn broad_phase(&mut self) {
        // TODO
    }
}