use crate::math_utils::{Rotation, Vec2};

pub struct Body {
    pub position: Vec2, // in the middle of body
    pub rotation: Rotation,

    pub velocity: Vec2,
    pub angular_velocity: f32,

    pub force: Vec2,
    pub torque: f32,

    pub width: Vec2,

    pub friction: f32,
    pub inv_mass: f32,
    pub inv_i: f32,
}

impl Body {
    /// Infinite mass: the body never moves.
    pub fn new_static(size: Vec2, position: Vec2) -> Self {
        Self::with_inverse_mass(size, position, 0.0, 0.0)
    }

    pub fn new_dynamic(size: Vec2, mass: f32, position: Vec2) -> Self {
        let i = mass * (size.x * size.x + size.y * size.y) / 12.0;
        Self::with_inverse_mass(size, position, 1.0 / mass, 1.0 / i)
    }

    fn with_inverse_mass(size: Vec2, position: Vec2, inv_mass: f32, inv_i: f32) -> Self {
        Self {
            position,
            rotation: Rotation::default(),
            velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            force: Vec2::ZERO,
            torque: 0.0,
            width: size,
            friction: 0.2,
            inv_mass,
            inv_i,
        }
    }

    pub fn is_static(&self) -> bool {
        self.inv_mass == 0.0
    }

    pub fn sub_velocity(&mut self, r: Vec2, p: Vec2) {
        self.velocity -= self.inv_mass * p;
        self.angular_velocity -= self.inv_i * r.cross(p);
    }

    pub fn add_velocity(&mut self, r: Vec2, p: Vec2) {
        self.velocity += self.inv_mass * p;
        self.angular_velocity += self.inv_i * r.cross(p);
    }

    pub fn add_force(&mut self, f: Vec2) {
        self.force += f;
    }
}
