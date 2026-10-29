use crate::bodies::{Bodies, BodyId};
use crate::body::Body;
use crate::math_utils::{Mat22, Vec2};
use crate::world::{POSITION_CORRECTION, WARM_STARTING};

const RIGID_BIAS_FACTOR: f32 = 0.2;
/// How a joint holds its two anchors together.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stiffness {
    /// The original correction: bias factor 0.2, no softness.
    Rigid,
    /// Spring with stiffness `k` and damping `d`, in world units.
    /// Softness and bias follow the step size, so the joint is the same at any rate.
    Spring { stiffness: f32, damping: f32 },
}

pub struct Joint {
    m: Mat22,
    local_anchor1: Vec2,
    local_anchor2: Vec2,
    r1: Vec2,
    r2: Vec2,
    bias: Vec2,
    p: Vec2, // accumulated impulse
    body1: BodyId,
    body2: BodyId,
    stiffness: Stiffness,
    // Derived from `stiffness` and the step size in `pre_step`.
    bias_factor: f32,
    softness: f32,
}

impl Joint {
    pub(crate) fn new(id1: BodyId, b1: &Body, id2: BodyId, b2: &Body, anchor: Vec2) -> Self {
        let local_anchor1 = b1.rotation.mat().transpose() * (anchor - b1.position);
        let local_anchor2 = b2.rotation.mat().transpose() * (anchor - b2.position);
        Self {
            m: Default::default(),
            local_anchor1,
            local_anchor2,
            r1: Vec2::ZERO,
            r2: Vec2::ZERO,
            bias: Vec2::ZERO,
            p: Vec2::ZERO, // accumulated impulse
            body1: id1,
            body2: id2,
            stiffness: Stiffness::Rigid,
            bias_factor: RIGID_BIAS_FACTOR,
            softness: 0.0,
        }
    }

    pub fn set_stiffness(&mut self, stiffness: Stiffness) {
        self.stiffness = stiffness;
    }

    pub fn involves(&self, id: BodyId) -> bool {
        self.body1 == id || self.body2 == id
    }

    pub(crate) fn lines(&self, bodies: &Bodies) -> [(Vec2, Vec2); 2] {
        let line = |id, local_anchor| {
            let body: &Body = bodies.get(id).expect("joint body is alive");
            (
                body.position,
                body.position + (body.rotation.mat() * local_anchor),
            )
        };
        [
            line(self.body1, self.local_anchor1),
            line(self.body2, self.local_anchor2),
        ]
    }

    pub(crate) fn pre_step(&mut self, bodies: &mut Bodies, inv_dt: f32) {
        match self.stiffness {
            Stiffness::Rigid => (self.softness, self.bias_factor) = (0.0, RIGID_BIAS_FACTOR),
            Stiffness::Spring {
                stiffness: k,
                damping: d,
            } if inv_dt > 0.0 => {
                let h = 1.0 / inv_dt;
                // Soft constraint, Catto GDC 2011: softness multiplies an impulse, hence 1/h.
                self.softness = 1.0 / (h * (d + h * k));
                self.bias_factor = h * k / (d + h * k);
            }
            // No time step: keep the last values.
            Stiffness::Spring { .. } => {}
        }
        let (body1, body2) = bodies.pair_mut(self.body1, self.body2);

        // Pre-compute anchors, mass matrix, and bias.
        self.r1 = body1.rotation.mat() * self.local_anchor1;
        self.r2 = body2.rotation.mat() * self.local_anchor2;

        // deltaV = deltaV0 + K * impulse
        // invM = [(1/m1 + 1/m2) * eye(2) - skew(r1) * invI1 * skew(r1) - skew(r2) * invI2 * skew(r2)]
        //      = [1/m1+1/m2     0    ] + invI1 * [r1.y*r1.y -r1.x*r1.y] + invI2 * [r1.y*r1.y -r1.x*r1.y]
        //        [    0     1/m1+1/m2]           [-r1.x*r1.y r1.x*r1.x]           [-r1.x*r1.y r1.x*r1.x]
        let inv_mass = body1.inv_mass + body2.inv_mass;
        let k1 = Mat22::new(Vec2::new(inv_mass, 0.0), Vec2::new(0.0, inv_mass));
        let skew = |inv_i: f32, r: Vec2| {
            Mat22::new(
                Vec2::new(inv_i * r.y * r.y, -inv_i * r.x * r.y),
                Vec2::new(-inv_i * r.x * r.y, inv_i * r.x * r.x),
            )
        };
        let k2 = skew(body1.inv_i, self.r1);
        let k3 = skew(body2.inv_i, self.r2);

        let mut k = (k1 + k2) + k3;
        k.col1.x += self.softness;
        k.col2.y += self.softness;
        self.m = k.invert();

        let p1 = body1.position + self.r1;
        let p2 = body2.position + self.r2;
        let dp = p2 - p1;
        self.bias = if POSITION_CORRECTION {
            -self.bias_factor * inv_dt * dp
        } else {
            Vec2::ZERO
        };

        if WARM_STARTING {
            // Apply accumulated impulse.
            body1.sub_velocity(self.r1, self.p);
            body2.add_velocity(self.r2, self.p);
        } else {
            self.p = Vec2::ZERO;
        }
    }

    pub(crate) fn apply_impulse(&mut self, bodies: &mut Bodies) {
        let (body1, body2) = bodies.pair_mut(self.body1, self.body2);
        let dv = (body2.velocity + Vec2::scalar_cross(body2.angular_velocity, self.r2))
            - (body1.velocity + Vec2::scalar_cross(body1.angular_velocity, self.r1));
        let impulse = self.m * ((self.bias - dv) - (self.softness * self.p));

        body1.sub_velocity(self.r1, impulse);
        body2.add_velocity(self.r2, impulse);
        self.p += impulse;
    }
}
