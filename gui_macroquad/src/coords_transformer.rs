use crate::size_params::SizeParams;
use physics::primitives::Rectangle;

pub fn rectangle_transform(r: &Rectangle, size_params: &SizeParams) -> (f32, f32, f32, f32) {
    let (x, y) = transform_coords(r.center.x, r.center.y, size_params);
    (
        x,
        y,
        r.width * size_params.width / 2.0,
        r.height * size_params.width / 2.0,
    )
}

pub fn transform_coords(x: f32, y: f32, size_params: &SizeParams) -> (f32, f32) {
    (
        size_params.offset_x + (x + 1.0) / 2.0 * size_params.width,
        size_params.offset_y + size_params.width - (y + 1.0) / 2.0 * size_params.width,
    )
}

pub fn transform_coords_back(x: f32, y: f32, size_params: &SizeParams) -> (f32, f32) {
    (
        (x - size_params.offset_x) / size_params.width * 2.0 - 1.0,
        (-y + size_params.offset_y + size_params.width) * 2.0 / size_params.width - 1.0,
    )
}
