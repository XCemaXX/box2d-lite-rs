use super::math_utils::Vec2;

pub struct Body {
	pub position: Vec2,
    pub rotation: f32,

	pub velocity: Vec2,
    pub angular_velocity: f32,

	pub force: Vec2,
	pub torque: f32,

	pub width: Vec2,

	friction: f32,
    mass: f32,
	pub inv_mass: f32,
    i: f32,
    pub inv_i: f32,
}

impl Body {
    pub fn default() -> Self {
        Self {
            position: Vec2::default(),
            rotation: 0.0,
            velocity: Vec2::default(),
            angular_velocity: 0.0,
            force: Vec2::default(),
            torque: 0.0,
            width: Vec2::new(1.0, 1.0),
            friction: 0.0,
            mass: f32::MAX,
            inv_mass: 0.0,
            i: f32::MAX,
            inv_i: 0.0,
        }
    }

    pub fn set(&mut self, w: Vec2, m: f32) {
        self.position.set(0.0, 0.0);
        self.rotation = 0.0;
        self.velocity.set(0.0, 0.0);
        self.angular_velocity = 0.0;
        self.force.set(0.0, 0.0);
        self.torque = 0.0;
        self.friction = 0.2;

        self.width = w;
        self.mass = m;

        if self.mass < f32::MAX {
            self.inv_mass = 1.0 / self.mass;
            self.i = self.mass * (self.width.x * self.width.x + self.width.y * self.width.y) / 12.0;
            self.inv_i = 1.0 / self.i;
        }
        else {
            self.inv_mass = 0.0;
            self.i = f32::MAX;
            self.inv_i = 0.0;
        }
    }

	pub fn add_force(&mut self, f: Vec2) {
		self.force += f;
	}
}