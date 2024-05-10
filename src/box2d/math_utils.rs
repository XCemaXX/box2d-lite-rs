
#[derive(Debug, Copy, Clone)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Default for Vec2 {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn set(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn abs(&self) -> Self {
        Self::new(f32::abs(self.x), f32::abs(self.y))
    }
}

impl std::ops::AddAssign<&Vec2> for Vec2 {
    fn add_assign(&mut self, other: &Vec2) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl std::ops::SubAssign<&Vec2> for Vec2 {
    fn sub_assign(&mut self, other: &Vec2) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl std::ops::Add for &Vec2 {
    type Output = Vec2;
    fn add(self, other: Self) -> Self::Output {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl std::ops::Sub for &Vec2 {
    type Output = Vec2;
    fn sub(self, other: Self) -> Self::Output {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}

pub fn dot(a: &Vec2, b: &Vec2) -> f32 {
	a.x * b.x + a.y * b.y
}

pub fn cross_v_v(a: &Vec2, b: &Vec2) -> f32 {
	a.x * b.y - a.y * b.x
}

pub fn cross_v_f(a: &Vec2, s: f32) -> Vec2 {
	Vec2::new(s * a.y, -s * a.x)
}

pub fn cross_f_v(s: f32, a: &Vec2) -> Vec2 {
	Vec2::new(-s * a.y, s * a.x)
}

impl std::ops::Neg for &Vec2 {
    type Output = Vec2;
    fn neg(self) -> Self::Output {
        Vec2::new(-self.x, -self.y)
    }
}

// ###########f32
impl std::ops::Mul<f32> for &Vec2 {
    type Output = Vec2;
    fn mul(self, other: f32) -> Self::Output {
        Vec2::new(self.x * other, self.y * other)
    }
} 

impl std::ops::Mul<&Vec2> for f32 {
    type Output = Vec2;
    fn mul(self, other: &Vec2) -> Self::Output {
        Vec2::new(self * other.x, self * other.y)
    }
}

// #################mat22
pub struct Mat22 {
    pub col1: Vec2,
    pub col2: Vec2,
}

impl Mat22 {
    pub fn new(col1: Vec2, col2: Vec2) -> Self {
        Self { col1, col2 }
    }

    pub fn from_angle(angle: f32) -> Self {
        let c = f32::cos(angle);
        let s = f32::sin(angle);
        Self {
            col1: Vec2::new(c, s),
            col2: Vec2::new(-s, c),
        }
    }

    pub fn transpose(&self) -> Self {
        Self::new(
            Vec2::new(self.col1.x, self.col2.x), 
            Vec2::new(self.col1.y, self.col2.y)
        )
    }

    pub fn abs(&self) -> Self {
        Self::new(
            self.col1.abs(),
            self.col2.abs()
        )
    }
}

impl std::ops::Mul for &Mat22 {
    type Output = Mat22;
    fn mul(self, other: Self) -> Self::Output {
        Mat22::new(self * &other.col1, self * &other.col2)
    }
}

//################ Combined

impl std::ops::Mul<&Vec2> for &Mat22 {
    type Output = Vec2;

    fn mul(self, v: &Vec2) -> Self::Output {
        Vec2::new(
            self.col1.x * v.x + self.col2.x * v.y,
            self.col1.y * v.x + self.col2.y * v.y
        )
    }
}

pub fn clamp(a: f32, low: f32, high: f32) -> f32 {
	f32::max(low, f32::min(a, high))
}