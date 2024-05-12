pub struct Point {
    pub x: f32,
    pub y: f32,
}

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