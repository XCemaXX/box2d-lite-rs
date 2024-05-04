
#[derive(Copy, Clone)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

pub struct Mat22 {
    col1: Vec2,
    col2: Vec2,
}

impl Vec2 {
    pub fn default() -> Self {
        Self {
            x: 0.0, 
            y: 0.0,
        }
    }
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x, 
            y,
        }
    }
    pub fn set(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }
}

impl std::ops::AddAssign for Vec2 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, other: f32) -> Self::Output {
        Vec2::new(self.x * other, self.y * other)
    }
} 

impl std::ops::Mul<Vec2> for f32 {
    type Output = Vec2;
    fn mul(self, other: Vec2) -> Self::Output {
        Vec2::new(self * other.x, self * other.y)
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl Mat22 {
    pub fn new(angle: f32) -> Self {
        let c = f32::cos(angle);
        let s = f32::sin(angle);
        Self {
            col1: Vec2::new(c, s),
            col2: Vec2::new(-s, c),
        }
    }
    /*
	Mat22(const Vec2& col1, const Vec2& col2) : col1(col1), col2(col2) {}

	Mat22 Transpose() const
	{
		return Mat22(Vec2(col1.x, col2.x), Vec2(col1.y, col2.y));
	}

	Mat22 Invert() const
	{
		float a = col1.x, b = col2.x, c = col1.y, d = col2.y;
		Mat22 B;
		float det = a * d - b * c;
		assert(det != 0.0f);
		det = 1.0f / det;
		B.col1.x =  det * d;	B.col2.x = -det * b;
		B.col1.y = -det * c;	B.col2.y =  det * a;
		return B;
	}*/
}

/*impl std::ops::Add<Mat22> for Vec2 {
    type Output = Vec2;
    fn add(self, other: Mat22) -> Vec2 {

    }
}*/