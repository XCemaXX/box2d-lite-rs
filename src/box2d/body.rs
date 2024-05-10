use super::math_utils::{Vec2, cross_v_v};

pub const UNMOVABLE_MASS: f32 = f32::MAX;

pub struct Body {
	pub position: Vec2, // in the middle of body
    pub rotation: f32,

	pub velocity: Vec2,
    pub angular_velocity: f32,

	pub force: Vec2,
	pub torque: f32,

	pub width: Vec2,

	pub friction: f32,
    mass: f32,
	pub inv_mass: f32,
    i: f32,
    pub inv_i: f32,

    pub serial_number: usize, // for arbiter instead of compare addresses
}

impl Default for Body {
    fn default() -> Self {
        Self { 
            position: Default::default(),
            rotation: 0.0,
            velocity: Default::default(),
            angular_velocity: 0.0,
            force: Default::default(),
            torque: 0.0,
            width: Vec2::new(1.0, 1.0),
            friction: 0.0,
            mass: UNMOVABLE_MASS,
            inv_mass: 0.0,
            i: UNMOVABLE_MASS,
            inv_i: 0.0,
            serial_number: 0,
        }
    }
}

impl Body {
    pub fn new(w: f32, h: f32, mass: f32, pos_x: f32, pos_y: f32) -> Self {
        let mut body = Self::default();
        body.set(Vec2::new(w, h), mass);
        body.position.set(pos_x, pos_y);
        body
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

        if self.mass < UNMOVABLE_MASS {
            self.inv_mass = 1.0 / self.mass;
            self.i = self.mass * (self.width.x * self.width.x + self.width.y * self.width.y) / 12.0;
            self.inv_i = 1.0 / self.i;
        }
        else {
            self.inv_mass = 0.0;
            self.i = UNMOVABLE_MASS;
            self.inv_i = 0.0;
        }
    }


    pub fn sub_velocity(&mut self, r: &Vec2, p: &Vec2) {
        self.velocity -= &(self.inv_mass * p);
        self.angular_velocity -= self.inv_i * cross_v_v(r, p);
    }

    pub fn add_velocity(&mut self, r: &Vec2, p: &Vec2) {
        self.velocity += &(self.inv_mass * p);
        self.angular_velocity += self.inv_i * cross_v_v(r, p);
    }

    pub fn add_force(&mut self, f: &Vec2) {
		self.force += f;
	}
}