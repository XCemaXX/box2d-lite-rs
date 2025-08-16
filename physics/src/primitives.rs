#[derive(Debug, Copy, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Into<[f32; 2]> for Point {
    fn into(self) -> [f32; 2] {
        [self.x, self.y]
    }
}

#[derive(Debug, Clone)]
pub struct Rectangle {
    pub center: Point,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
}

pub struct Line {
    pub p1: Point,
    pub p2: Point,
}

pub struct Triangle {
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
}
