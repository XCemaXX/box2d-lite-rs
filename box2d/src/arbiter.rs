use crate::body::Body;
use crate::collide::collide;
use crate::contact::{Contact, MAX_CONTACT_POINT};
use crate::math_utils::{Vec2, clamp, cross_f_v, cross_v_f, dot};
use crate::world::{ACCUMULATE_IMPULSES, POSITION_CORRECTION, WARM_STARTING};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Arbiter {
    contacts: [Contact; MAX_CONTACT_POINT],
    num_contacts: usize,
    body1: Rc<RefCell<Body>>,
    body2: Rc<RefCell<Body>>,
    // Combined friction
    friction: f32,
}

impl Arbiter {
    pub fn create_arbiter_contacts(b1: Rc<RefCell<Body>>, b2: Rc<RefCell<Body>>) -> (Self, usize) {
        let (b1, b2) = if b1.borrow().serial_number < b2.borrow().serial_number {
            (b1, b2)
        } else {
            (b2, b1)
        };
        let mut contacts: [Contact; MAX_CONTACT_POINT] = Default::default();
        let body1 = b1.clone();
        let body1 = &body1.borrow();
        let body2 = b2.clone();
        let body2 = &body2.borrow();

        let num_contacts = collide(&mut contacts, body1, body2);
        let friction = f32::sqrt(body1.friction * body2.friction);
        (
            Self {
                contacts,
                num_contacts,
                body1: b1,
                body2: b2,
                friction,
            },
            num_contacts,
        )
    }

    pub fn get_collide_points(&self) -> Vec<Vec2> {
        let mut res = Vec::new();
        for i in 0..self.num_contacts {
            res.push(Vec2::new(
                self.contacts[i].position.x,
                self.contacts[i].position.y,
            ));
        }
        res
    }

    pub fn update(&mut self, other: &Self) {
        self._update(&other.contacts, other.num_contacts);
    }

    fn _update(&mut self, new_contacts: &[Contact; MAX_CONTACT_POINT], num_new_contacts: usize) {
        let mut merged_contacts: [Contact; MAX_CONTACT_POINT] = Default::default();

        for i in 0..num_new_contacts {
            let c_new = &new_contacts[i];

            let k = 'found: {
                for j in 0..self.num_contacts {
                    let c_old = &self.contacts[j];
                    if c_new.feature == c_old.feature {
                        break 'found Some(j);
                    }
                }
                None
            };

            if let Some(k) = k {
                let c_old = &self.contacts[k];
                merged_contacts[i] = c_new.clone();
                if WARM_STARTING {
                    merged_contacts[i].pn = c_old.pn;
                    merged_contacts[i].pt = c_old.pt;
                    merged_contacts[i].pnb = c_old.pnb;
                } else {
                    merged_contacts[i].pn = 0.0;
                    merged_contacts[i].pt = 0.0;
                    merged_contacts[i].pnb = 0.0;
                }
            } else {
                merged_contacts[i] = new_contacts[i].clone();
            }
        }
        self.contacts = merged_contacts;
        self.num_contacts = num_new_contacts;
    }

    pub fn pre_step(&mut self, inv_dt: f32) {
        const K_ALLOWED_PENETRATION: f32 = 0.01;
        let k_bias_factor = if POSITION_CORRECTION { 0.2 } else { 0.0 };

        for i in 0..self.num_contacts {
            let c = &mut self.contacts[i];
            let (tangent, r1, r2) = {
                let body1 = self.body1.borrow();
                let body2 = self.body2.borrow();

                let r1 = c.position - body1.position;
                let r2 = c.position - body2.position;

                // Precompute normal mass, tangent mass, and bias.
                let rn1 = dot(r1, c.normal);
                let rn2 = dot(r2, c.normal);
                let k_normal = body1.inv_mass
                    + body2.inv_mass
                    + body1.inv_i * (dot(r1, r1) - rn1 * rn1)
                    + body2.inv_i * (dot(r2, r2) - rn2 * rn2);
                c.mass_normal = 1.0 / k_normal;

                let tangent = cross_v_f(c.normal, 1.0);
                let rt1 = dot(r1, tangent);
                let rt2 = dot(r2, tangent);
                let k_tangent = body1.inv_mass
                    + body2.inv_mass
                    + body1.inv_i * (dot(r1, r1) - rt1 * rt1)
                    + body2.inv_i * (dot(r2, r2) - rt2 * rt2);
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
        for i in 0..self.num_contacts {
            let c = &mut self.contacts[i];

            let (dpn, pn) = {
                let body1 = self.body1.borrow();
                let body2 = self.body2.borrow();
                c.r1 = c.position - body1.position;
                c.r2 = c.position - body2.position;

                // Relative velocity at contact
                let dv = (body2.velocity + cross_f_v(body2.angular_velocity, c.r2))
                    - (body1.velocity + cross_f_v(body1.angular_velocity, c.r1));

                // Compute normal impulse
                let vn = dot(dv, c.normal);

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
                let dv = (body2.velocity + cross_f_v(body2.angular_velocity, c.r2))
                    - (body1.velocity + cross_f_v(body1.angular_velocity, c.r1));

                let tangent = cross_v_f(c.normal, 1.0);
                let vt = dot(dv, tangent);
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
