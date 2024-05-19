use crate::primitives::Rectangle;

struct Vec2 {
    pub x: f32,
    pub y: f32,
}
struct Mat22 {
    pub col1: Vec2,
    pub col2: Vec2,
}

impl Mat22 {
    fn from_angle(angle: f32) -> Self {
        let c = f32::cos(angle);
        let s = f32::sin(angle);
        Self {
            col1: Vec2{x: c, y: s},
            col2: Vec2{x: -s, y: c},
        }
    }
}

impl std::ops::Mul<&Vec2> for &Mat22 {
    type Output = Vec2;
    fn mul(self, v: &Vec2) -> Self::Output {
        Vec2{
            x: self.col1.x * v.x + self.col2.x * v.y,
            y: self.col1.y * v.x + self.col2.y * v.y
        }
    }
}

impl std::ops::Mul<f32> for &Vec2 {
    type Output = Vec2;
    fn mul(self, other: f32) -> Self::Output {
        Vec2{ x: self.x * other, y: self.y * other }
    }
} 

impl std::ops::Add for &Vec2 {
    type Output = Vec2;
    fn add(self, other: Self) -> Self::Output {
        Vec2{ x: self.x + other.x, y: self.y + other.y}
    }
}

impl Into<[f32; 2]> for Vec2 {
    fn into(self) -> [f32; 2] {
        [self.x, self.y] 
    }
}

pub fn get_corners(rect: &Rectangle) -> [[f32; 2]; 4] {
    let r = Mat22::from_angle(rect.rotation);
    let center = Vec2{x: rect.center.x, y: rect.center.y};
    let w = &Vec2{x: rect.width, y: rect.height} * 0.5;

    let left_bot:  [f32; 2] = (&center + &(&r * &Vec2{x: -w.x, y: -w.y})).into();
    let right_bot: [f32; 2] = (&center + &(&r * &Vec2{x:  w.x, y: -w.y})).into();
    let right_top: [f32; 2] = (&center + &(&r * &Vec2{x:  w.x, y:  w.y})).into();
    let left_top:  [f32; 2] = (&center + &(&r * &Vec2{x: -w.x, y:  w.y})).into(); 

    [left_bot, right_bot, right_top, left_top]
}