#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2::new(0.0, 0.0);

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs())
    }

    pub fn dot(self, b: Vec2) -> f32 {
        self.x * b.x + self.y * b.y
    }

    /// 2D cross product: the z of the 3D one.
    pub fn cross(self, b: Vec2) -> f32 {
        self.x * b.y - self.y * b.x
    }

    /// `self × s`, with `s` along z.
    pub fn cross_scalar(self, s: f32) -> Vec2 {
        Vec2::new(s * self.y, -s * self.x)
    }

    /// `s × v`, with `s` along z.
    pub fn scalar_cross(s: f32, v: Vec2) -> Vec2 {
        Vec2::new(-s * v.y, s * v.x)
    }
}

impl std::ops::AddAssign for Vec2 {
    fn add_assign(&mut self, other: Vec2) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl std::ops::SubAssign for Vec2 {
    fn sub_assign(&mut self, other: Vec2) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;
    fn add(self, other: Vec2) -> Self::Output {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, other: Vec2) -> Self::Output {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}

impl std::ops::Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Self::Output {
        Vec2::new(-self.x, -self.y)
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, s: f32) -> Self::Output {
        Vec2::new(self.x * s, self.y * s)
    }
}

impl std::ops::Mul<Vec2> for f32 {
    type Output = Vec2;
    fn mul(self, v: Vec2) -> Self::Output {
        Vec2::new(self * v.x, self * v.y)
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Mat22 {
    pub col1: Vec2,
    pub col2: Vec2,
}

impl Mat22 {
    pub const fn new(col1: Vec2, col2: Vec2) -> Self {
        Self { col1, col2 }
    }

    /// Rotation by `angle` radians.
    pub fn from_angle(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            col1: Vec2::new(c, s),
            col2: Vec2::new(-s, c),
        }
    }

    pub fn transpose(self) -> Self {
        Self::new(
            Vec2::new(self.col1.x, self.col2.x),
            Vec2::new(self.col1.y, self.col2.y),
        )
    }

    pub fn abs(self) -> Self {
        Self::new(self.col1.abs(), self.col2.abs())
    }

    pub fn invert(self) -> Self {
        let (a, b, c, d) = (self.col1.x, self.col2.x, self.col1.y, self.col2.y);
        let det = a * d - b * c;
        debug_assert!(det != 0.0, "singular matrix");
        let det = 1.0 / det;

        Mat22::new(Vec2::new(det * d, -det * c), Vec2::new(-det * b, det * a))
    }
}

impl std::ops::Mul for Mat22 {
    type Output = Mat22;
    fn mul(self, other: Mat22) -> Self::Output {
        Mat22::new(self * other.col1, self * other.col2)
    }
}

impl std::ops::Add for Mat22 {
    type Output = Mat22;
    fn add(self, other: Mat22) -> Self::Output {
        Mat22::new(self.col1 + other.col1, self.col2 + other.col2)
    }
}

impl std::ops::Mul<Vec2> for Mat22 {
    type Output = Vec2;

    fn mul(self, v: Vec2) -> Self::Output {
        Vec2::new(
            self.col1.x * v.x + self.col2.x * v.y,
            self.col1.y * v.x + self.col2.y * v.y,
        )
    }
}

/// Same as `Clamp` in the original. Not `f32::clamp`: it panics when `low > high`
/// and returns NaN for a NaN input, where this returns `high`.
pub fn clamp(a: f32, low: f32, high: f32) -> f32 {
    f32::max(low, f32::min(a, high))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Vec2, b: Vec2) -> bool {
        (a - b).abs().x < 1e-6 && (a - b).abs().y < 1e-6
    }

    #[test]
    fn rotation_inverse_is_transpose() {
        let r = Mat22::from_angle(0.7);
        let i = r * r.invert();
        assert!(close(i.col1, Vec2::new(1.0, 0.0)) && close(i.col2, Vec2::new(0.0, 1.0)));
        let t = r * r.transpose();
        assert!(close(t.col1, Vec2::new(1.0, 0.0)) && close(t.col2, Vec2::new(0.0, 1.0)));
    }

    #[test]
    fn cross_products() {
        let (a, b) = (Vec2::new(2.0, 3.0), Vec2::new(-1.0, 4.0));
        assert_eq!(a.cross(b), -b.cross(a));
        assert_eq!(a.cross(a), 0.0);
        // v × s and s × v point in opposite directions, both perpendicular to v.
        assert_eq!(a.cross_scalar(2.0), -Vec2::scalar_cross(2.0, a));
        assert_eq!(a.dot(a.cross_scalar(2.0)), 0.0);
    }

    #[test]
    fn clamp_keeps_original_semantics() {
        assert_eq!(clamp(5.0, -1.0, 1.0), 1.0);
        assert_eq!(clamp(-5.0, -1.0, 1.0), -1.0);
        assert_eq!(clamp(f32::NAN, -1.0, 1.0), 1.0);
    }
}
