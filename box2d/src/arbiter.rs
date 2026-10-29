use crate::body::Body;
use crate::collide::collide;
use crate::contact::Contacts;
use crate::math_utils::{Vec2, clamp};
use crate::world::{ACCUMULATE_IMPULSES, POSITION_CORRECTION, WARM_STARTING};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Arbiter {
    contacts: Contacts,
    body1: Rc<RefCell<Body>>,
    body2: Rc<RefCell<Body>>,
    // Combined friction
    friction: f32,
}

impl Arbiter {
    pub fn new(b1: Rc<RefCell<Body>>, b2: Rc<RefCell<Body>>) -> Self {
        let (body1, body2) = if b1.borrow().serial_number < b2.borrow().serial_number {
            (b1, b2)
        } else {
            (b2, b1)
        };
        let (contacts, friction) = {
            let (b1, b2) = (body1.borrow(), body2.borrow());
            (collide(&b1, &b2), f32::sqrt(b1.friction * b2.friction))
        };
        Self {
            contacts,
            body1,
            body2,
            friction,
        }
    }

    pub fn has_contacts(&self) -> bool {
        !self.contacts.is_empty()
    }

    pub fn collide_points(&self) -> impl Iterator<Item = Vec2> + '_ {
        self.contacts.as_slice().iter().map(|c| c.position)
    }

    /// Takes the new contacts; contacts with the same feature keep their impulses.
    pub fn update(&mut self, other: &Self) {
        let mut merged = other.contacts.clone();
        for c in merged.as_mut_slice() {
            let old = self
                .contacts
                .as_slice()
                .iter()
                .find(|old| old.feature == c.feature);
            if let Some(old) = old {
                if WARM_STARTING {
                    (c.pn, c.pt, c.pnb) = (old.pn, old.pt, old.pnb);
                } else {
                    (c.pn, c.pt, c.pnb) = (0.0, 0.0, 0.0);
                }
            }
        }
        self.contacts = merged;
    }

    pub fn pre_step(&mut self, inv_dt: f32) {
        const K_ALLOWED_PENETRATION: f32 = 0.01;
        let k_bias_factor = if POSITION_CORRECTION { 0.2 } else { 0.0 };

        for c in self.contacts.as_mut_slice() {
            let (tangent, r1, r2) = {
                let body1 = self.body1.borrow();
                let body2 = self.body2.borrow();

                let r1 = c.position - body1.position;
                let r2 = c.position - body2.position;

                // Precompute normal mass, tangent mass, and bias.
                let rn1 = r1.dot(c.normal);
                let rn2 = r2.dot(c.normal);
                let k_normal = body1.inv_mass
                    + body2.inv_mass
                    + body1.inv_i * (r1.dot(r1) - rn1 * rn1)
                    + body2.inv_i * (r2.dot(r2) - rn2 * rn2);
                c.mass_normal = 1.0 / k_normal;

                let tangent = c.normal.cross_scalar(1.0);
                let rt1 = r1.dot(tangent);
                let rt2 = r2.dot(tangent);
                let k_tangent = body1.inv_mass
                    + body2.inv_mass
                    + body1.inv_i * (r1.dot(r1) - rt1 * rt1)
                    + body2.inv_i * (r2.dot(r2) - rt2 * rt2);
                c.mass_tangent = 1.0 / k_tangent;

                c.bias =
                    -k_bias_factor * inv_dt * f32::min(0.0, c.separation + K_ALLOWED_PENETRATION);
                (tangent, r1, r2)
            };

            if ACCUMULATE_IMPULSES {
                // Apply normal + friction impulse
                let p = (c.pn * c.normal) + (c.pt * tangent);
                self.body1.borrow_mut().sub_velocity(r1, p);
                self.body2.borrow_mut().add_velocity(r2, p);
            }
        }
    }

    pub fn apply_impulse(&mut self) {
        for c in self.contacts.as_mut_slice() {
            let (dpn, pn) = {
                let body1 = self.body1.borrow();
                let body2 = self.body2.borrow();
                c.r1 = c.position - body1.position;
                c.r2 = c.position - body2.position;

                // Relative velocity at contact
                let dv = (body2.velocity + Vec2::scalar_cross(body2.angular_velocity, c.r2))
                    - (body1.velocity + Vec2::scalar_cross(body1.angular_velocity, c.r1));

                // Compute normal impulse
                let vn = dv.dot(c.normal);

                let dpn = c.mass_normal * (-vn + c.bias);
                let dpn = if ACCUMULATE_IMPULSES {
                    // Clamp the accumulated impulse
                    let pn0 = c.pn;
                    c.pn = f32::max(pn0 + dpn, 0.0);
                    c.pn - pn0
                } else {
                    f32::max(dpn, 0.0)
                };

                // Apply contact impulse
                (dpn, dpn * c.normal)
            };
            self.body1.borrow_mut().sub_velocity(c.r1, pn);
            self.body2.borrow_mut().add_velocity(c.r2, pn);

            let pt = {
                let body1 = self.body1.borrow();
                let body2 = self.body2.borrow();
                // Relative velocity at contact
                let dv = (body2.velocity + Vec2::scalar_cross(body2.angular_velocity, c.r2))
                    - (body1.velocity + Vec2::scalar_cross(body1.angular_velocity, c.r1));

                let tangent = c.normal.cross_scalar(1.0);
                let vt = dv.dot(tangent);
                let dpt = c.mass_tangent * (-vt);

                let dpt = if ACCUMULATE_IMPULSES {
                    // Compute friction impulse
                    let max_pt = self.friction * c.pn;

                    // Clamp friction
                    let old_tangent_impulse = c.pt;
                    c.pt = clamp(old_tangent_impulse + dpt, -max_pt, max_pt);
                    c.pt - old_tangent_impulse
                } else {
                    let max_pt = self.friction * dpn;
                    clamp(dpt, -max_pt, max_pt)
                };
                // Apply contact impulse
                dpt * tangent
            };
            self.body1.borrow_mut().sub_velocity(c.r1, pt);
            self.body2.borrow_mut().add_velocity(c.r2, pt);
        }
    }

    pub fn get_key(&self) -> (usize, usize) {
        (
            self.body1.borrow().serial_number,
            self.body2.borrow().serial_number,
        )
    }
}
