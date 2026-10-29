use crate::arbiter::Arbiter;
use crate::body::Body;
use crate::joint::Joint;
use crate::math_utils::Vec2;

use std::cell::{Ref, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;

pub struct World {
    bodies: Vec<Rc<RefCell<Body>>>,
    joints: Vec<Rc<RefCell<Joint>>>,
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
            joints: Vec::new(),
            arbiters: BTreeMap::new(),
            gravity,
            iterations,
        }
    }

    pub fn bodies(&self) -> impl Iterator<Item = Ref<'_, Body>> {
        self.bodies.iter().map(|b| b.borrow())
    }

    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    pub fn collide_points(&self) -> impl Iterator<Item = Vec2> + '_ {
        self.arbiters.values().flat_map(Arbiter::collide_points)
    }

    /// Two lines per joint: from each body center to the anchor.
    pub fn joint_lines(&self) -> impl Iterator<Item = (Vec2, Vec2)> + '_ {
        self.joints.iter().flat_map(|j| j.borrow().lines())
    }

    pub fn add_body(&mut self, body: Rc<RefCell<Body>>) {
        body.borrow_mut().serial_number = self.bodies.len();
        self.bodies.push(body);
    }

    pub fn add_joint(
        &mut self,
        body1: Rc<RefCell<Body>>,
        body2: Rc<RefCell<Body>>,
        anchor: Vec2,
    ) -> Rc<RefCell<Joint>> {
        let joint = Rc::new(RefCell::new(Joint::new(body1, body2, anchor)));
        self.joints.push(joint.clone());
        joint
    }

    pub fn clear(&mut self) {
        self.bodies.clear();
        self.joints.clear();
        self.arbiters.clear();
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
            body.velocity += dt * (self.gravity + (inv_mass * force));
            body.angular_velocity += dt * body.inv_i * body.torque;
        }

        // Perform pre-steps.
        for arb in self.arbiters.values_mut() {
            arb.pre_step(inv_dt);
        }

        for joint in &mut self.joints {
            let mut joint = joint.borrow_mut();
            joint.pre_step(inv_dt);
        }

        // Perform iterations
        for _ in 0..self.iterations {
            for arb in self.arbiters.values_mut() {
                arb.apply_impulse();
            }
            for joint in &mut self.joints {
                let mut joint = joint.borrow_mut();
                joint.apply_impulse();
            }
        }

        // Integrate Velocities
        for body in &mut self.bodies {
            let velocity = body.borrow().velocity;
            let angular_velocity = body.borrow().angular_velocity;
            let mut body = body.borrow_mut();

            body.position += dt * velocity;
            body.rotation += dt * angular_velocity;

            body.force.set(0.0, 0.0);
            body.torque = 0.0;
        }
    }

    fn broad_phase(&mut self) {
        // O(n^2) broad-phase
        for i in 0..self.bodies.len() {
            let bi = self.bodies[i].borrow();
            for j in (i + 1)..self.bodies.len() {
                let bj = self.bodies[j].borrow();

                if bi.inv_mass == 0.0 && bj.inv_mass == 0.0 {
                    continue;
                }
                let (new_arb, new_num_contacts) = Arbiter::create_arbiter_contacts(
                    self.bodies[i].clone(),
                    self.bodies[j].clone(),
                );
                let key = new_arb.get_key();

                if new_num_contacts > 0 {
                    if let Some(arb) = self.arbiters.get_mut(&key) {
                        arb.update(&new_arb);
                    } else {
                        self.arbiters.insert(key, new_arb);
                    }
                } else {
                    self.arbiters.remove(&key);
                }
            }
        }
    }
}
