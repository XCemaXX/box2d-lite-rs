#[derive(Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct Rectangle {
    pub center: Point,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
}