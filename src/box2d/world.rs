use super::math_utils::Vec2;
use super::body::Body;
use super::arbiter::Arbiter;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::BTreeMap;

pub struct World {
    bodies: Vec<Rc<RefCell<Body>>>,
    arbiters: BTreeMap<(usize, usize), Arbiter>,
    gravity: Vec2,
    iterations: i32,
}

pub static ACCUMULATE_IMPULSES: bool = true;
pub static WARM_STARTING: bool = true;
pub static POSITION_CORRECTION: bool = true;

impl World {
    pub fn new(gravity: Vec2, iterations: i32) -> Self {
        Self {
            bodies: Vec::new(),
            arbiters: BTreeMap::new(),
            gravity,
            iterations,
        }
    }

    pub fn get_collide_points(&self) -> Vec<Vec2> {
        let mut res = Vec::new();
        for (_, arb) in &self.arbiters {
            res.extend(arb.get_collide_points().iter());
        }
        res
    }

    pub fn add_body(&mut self, body: Rc<RefCell<Body>>) {
        body.borrow_mut().serial_number = self.bodies.len();
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

            if inv_mass == 0.0 {
                continue;
            }
            body.velocity += &(dt * &(&self.gravity + &(inv_mass * &force)));
            body.angular_velocity += dt * body.inv_i * body.torque;
        }
        
        // Perform pre-steps.
        for (_, arb) in &mut self.arbiters {
            arb.pre_step(inv_dt);
        }
        // todo
        /*for (int i = 0; i < (int)joints.size(); ++i) {
            joints[i]->PreStep(inv_dt);	
        }*/

        // Perform iterations
        for _ in 0..self.iterations {
            for (_, arb) in &mut self.arbiters {
                arb.apply_impulse();
            }
            // todo
            /*for (int j = 0; j < (int)joints.size(); ++j)
            {
                joints[j]->ApplyImpulse();
            }*/
        }
        
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
        // O(n^2) broad-phase
        for i in 0..self.bodies.len() {
            let bi = self.bodies[i].borrow();
            for j in (i+1)..self.bodies.len() {
                let bj = self.bodies[j].borrow();

                if bi.inv_mass == 0.0 && bj.inv_mass == 0.0 {
                    continue;
                }
                let (new_arb, new_num_contacts) = Arbiter::create_arbiter_contacts(self.bodies[i].clone(), self.bodies[j].clone());
                let key = new_arb.get_key();

                if new_num_contacts > 0 {
                    if let Some(arb) = self.arbiters.get_mut(&key) {
                        arb.update(&new_arb);
                    } else {
                        self.arbiters.insert(key, new_arb);
                    }
                }
                else {
                    self.arbiters.remove(&key);
                }
            }
        }
    }
}