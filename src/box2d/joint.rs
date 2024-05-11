
use super::math_utils::{Vec2, Mat22, cross_f_v};
use super::body::Body;
use super::world::{WARM_STARTING, POSITION_CORRECTION};
use std::rc::Rc;
use std::cell::RefCell;
pub struct Joint {
    m: Mat22,
    local_anchor1: Vec2,
    local_anchor2: Vec2,
    r1: Vec2,
    r2: Vec2,
    bias: Vec2,
    p: Vec2, // accumulated impulse
    body1: Rc<RefCell<Body>>,
    body2: Rc<RefCell<Body>>,
    bias_factor: f32,
    softness: f32,
}

impl Joint {
    pub fn new(body1: Rc<RefCell<Body>>, body2: Rc<RefCell<Body>>, anchor: &Vec2) -> Self {
        let (local_anchor1, local_anchor2) = {
            let body1 = &body1.borrow();
            let body2 = &body2.borrow();
        
            let rot1 = Mat22::from_angle(body1.rotation);
            let rot2 = Mat22::from_angle(body2.rotation);
            let rot1t = rot1.transpose();
            let rot2t = rot2.transpose();
        
            (&rot1t * &(anchor - &body1.position), 
            &rot2t * &(anchor - &body2.position))
        };
        
        Self {
            m: Default::default(),
            local_anchor1: local_anchor1,
            local_anchor2: local_anchor2,
            r1: Vec2::default(),
            r2: Vec2::default(),
            bias: Vec2::default(),
            p: Vec2::default(), // accumulated impulse
            body1: body1,
            body2: body2,
            bias_factor: 0.2,
            softness: 0.0,
        }
    }

    pub fn get_joint_lines(&self) -> [(Vec2, Vec2); 2] {
        let b1 = self.body1.borrow();
        let b2 = self.body2.borrow();
        let r1 = Mat22::from_angle(b1.rotation);
        let r2 = Mat22::from_angle(b2.rotation);
        let x1 = b1.position;
        let p1 = &x1 + &(&r1 * &self.local_anchor1);
        let x2 = b2.position;
        let p2 = &x2 + &(&r2 * &self.local_anchor2);

        [(x1, p1), (x2, p2)]
    }

	pub fn pre_step(&mut self, inv_dt: f32) {
        {
            let body1 = &self.body1.borrow();
            let body2 = &self.body2.borrow();
            // Pre-compute anchors, mass matrix, and bias.
            self.r1 = &Mat22::from_angle(body1.rotation) * &self.local_anchor1;
            self.r2 = &Mat22::from_angle(body2.rotation) * &self.local_anchor2;

            // deltaV = deltaV0 + K * impulse
            // invM = [(1/m1 + 1/m2) * eye(2) - skew(r1) * invI1 * skew(r1) - skew(r2) * invI2 * skew(r2)]
            //      = [1/m1+1/m2     0    ] + invI1 * [r1.y*r1.y -r1.x*r1.y] + invI2 * [r1.y*r1.y -r1.x*r1.y]
            //        [    0     1/m1+1/m2]           [-r1.x*r1.y r1.x*r1.x]           [-r1.x*r1.y r1.x*r1.x]
            let mut k1 = Mat22::default();
            (k1.col1.x, k1.col2.x) = ( body1.inv_mass + body2.inv_mass,  0.0);
            (k1.col1.y, k1.col2.y) = ( 0.0,                              body1.inv_mass + body2.inv_mass);

            let mut k2 = Mat22::default();
            (k2.col1.x, k2.col2.x) = ( body1.inv_i * self.r1.y * self.r1.y,        -body1.inv_i * self.r1.x * self.r1.y);
            (k2.col1.y, k2.col2.y) = (-body1.inv_i * self.r1.x * self.r1.y,         body1.inv_i * self.r1.x * self.r1.x);

            let mut k3 = Mat22::default();
            (k3.col1.x, k3.col2.x) = ( body2.inv_i * self.r2.y * self.r2.y,		 -body2.inv_i * self.r2.x * self.r2.y);
            (k3.col1.y, k3.col2.y) = (-body2.inv_i * self.r2.x * self.r2.y,		  body2.inv_i * self.r2.x * self.r2.x);

            let mut k = &(&k1 + &k2) + &k3;
            k.col1.x += self.softness;
            k.col2.y += self.softness;

            self.m = k.invert();

            let p1 = &body1.position + &self.r1;
            let p2 = &body2.position + &self.r2;
            let dp = &p2 - &p1;

            if POSITION_CORRECTION {
                self.bias = -self.bias_factor * inv_dt * &dp;
            }
            else {
                self.bias.set(0.0, 0.0);
            }
        }

        if WARM_STARTING {
            // Apply accumulated impulse.
            self.body1.borrow_mut().sub_velocity(&self.r1, &self.p);
            self.body2.borrow_mut().add_velocity(&self.r2, &self.p);
        }
        else {
            self.p.set(0.0, 0.0);
        }
    }

	pub fn apply_impulse(&mut self) {
        let impulse = {
            let body1 = self.body1.borrow();
            let body2 = self.body2.borrow();
            let dv = &(&body2.velocity + &cross_f_v(body2.angular_velocity, &self.r2))
                - &(&body1.velocity + &cross_f_v(body1.angular_velocity, &self.r1));

            &self.m * &(&(&self.bias - &dv) - &(self.softness * &self.p))
        };

        self.body1.borrow_mut().sub_velocity(&self.r1, &impulse);
        self.body2.borrow_mut().add_velocity(&self.r2, &impulse);
        self.p += &impulse;
    }
}