use crate::arbiter::Arbiter;
use crate::bodies::{Bodies, BodyId};
use crate::body::Body;
use crate::joint::Joint;
use crate::math_utils::{Rotation, Vec2};

use std::collections::BTreeMap;

pub struct World {
    bodies: Bodies,
    joints: Vec<Joint>,
    arbiters: BTreeMap<(BodyId, BodyId), Arbiter>,
    gravity: Vec2,
    iterations: u32,
}

pub const ACCUMULATE_IMPULSES: bool = true;
pub const WARM_STARTING: bool = true;
pub const POSITION_CORRECTION: bool = true;

impl World {
    pub fn new(gravity: Vec2, iterations: u32) -> Self {
        Self {
            bodies: Bodies::default(),
            joints: Vec::new(),
            arbiters: BTreeMap::new(),
            gravity,
            iterations,
        }
    }

    pub fn add_body(&mut self, body: Body) -> BodyId {
        self.bodies.insert(body)
    }

    /// Also removes the joints and contacts of this body.
    pub fn remove_body(&mut self, id: BodyId) -> Option<Body> {
        let body = self.bodies.remove(id)?;
        self.joints.retain(|j| !j.involves(id));
        self.arbiters.retain(|_, a| !a.involves(id));
        Some(body)
    }

    /// `None` when the body was removed.
    pub fn body(&self, id: BodyId) -> Option<&Body> {
        self.bodies.get(id)
    }

    pub fn body_mut(&mut self, id: BodyId) -> Option<&mut Body> {
        self.bodies.get_mut(id)
    }

    pub fn bodies(&self) -> impl Iterator<Item = &Body> {
        self.bodies.iter().map(|(_, body)| body)
    }

    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    /// `None` when an id is not a live body, or both ids are the same body.
    pub fn add_joint(&mut self, id1: BodyId, id2: BodyId, anchor: Vec2) -> Option<&mut Joint> {
        if id1 == id2 {
            return None;
        }
        let joint = Joint::new(
            id1,
            self.bodies.get(id1)?,
            id2,
            self.bodies.get(id2)?,
            anchor,
        );
        self.joints.push(joint);
        self.joints.last_mut()
    }

    pub fn collide_points(&self) -> impl Iterator<Item = Vec2> + '_ {
        self.arbiters.values().flat_map(Arbiter::collide_points)
    }

    /// Two lines per joint: from each body center to the anchor.
    pub fn joint_lines(&self) -> impl Iterator<Item = (Vec2, Vec2)> + '_ {
        self.joints.iter().flat_map(|j| j.lines(&self.bodies))
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
        for body in self.bodies.iter_mut() {
            if body.is_static() {
                continue;
            }
            let (inv_mass, force) = (body.inv_mass, body.force);
            body.velocity += dt * (self.gravity + (inv_mass * force));
            body.angular_velocity += dt * body.inv_i * body.torque;
        }

        // Perform pre-steps.
        for arb in self.arbiters.values_mut() {
            arb.pre_step(&mut self.bodies, inv_dt);
        }

        for joint in &mut self.joints {
            joint.pre_step(&mut self.bodies, inv_dt);
        }

        // Perform iterations
        for _ in 0..self.iterations {
            for arb in self.arbiters.values_mut() {
                arb.apply_impulse(&mut self.bodies);
            }
            for joint in &mut self.joints {
                joint.apply_impulse(&mut self.bodies);
            }
        }

        // Integrate Velocities
        for body in self.bodies.iter_mut() {
            body.position += dt * body.velocity;
            body.rotation = Rotation::new(body.rotation.angle() + dt * body.angular_velocity);

            body.force = Vec2::ZERO;
            body.torque = 0.0;
        }
    }

    fn broad_phase(&mut self) {
        // O(n^2) broad-phase
        for (n, (id1, b1)) in self.bodies.iter().enumerate() {
            for (id2, b2) in self.bodies.iter().skip(n + 1) {
                if b1.is_static() && b2.is_static() {
                    continue;
                }
                let new_arb = Arbiter::new(id1, b1, id2, b2);
                let key = new_arb.key();

                if new_arb.has_contacts() {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn world_with_joint() -> (World, BodyId, BodyId) {
        let mut world = World::new(Vec2::new(0.0, -10.0), 10);
        let a = world.add_body(Body::new_static(Vec2::new(10.0, 1.0), Vec2::ZERO));
        let b = world.add_body(Body::new_dynamic(
            Vec2::new(1.0, 1.0),
            1.0,
            Vec2::new(0.0, 0.9),
        ));
        assert!(world.add_joint(a, b, Vec2::new(0.0, 2.0)).is_some());
        world.step(1.0 / 60.0);
        (world, a, b)
    }

    #[test]
    fn remove_body_drops_its_joints_and_contacts() {
        let (mut world, _, b) = world_with_joint();
        assert!(world.collide_points().count() > 0);
        assert!(world.remove_body(b).is_some());
        assert_eq!(world.collide_points().count(), 0);
        assert_eq!(world.joint_lines().count(), 0);
        assert!(world.body(b).is_none());
        // The world keeps working without the removed body.
        world.step(1.0 / 60.0);
    }

    #[test]
    fn joint_needs_two_live_bodies() {
        let (mut world, a, b) = world_with_joint();
        assert!(world.add_joint(a, a, Vec2::ZERO).is_none());
        world.remove_body(b);
        assert!(world.add_joint(a, b, Vec2::ZERO).is_none());
    }
}
