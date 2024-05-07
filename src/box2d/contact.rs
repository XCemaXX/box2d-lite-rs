use super::math_utils::Vec2;

// Box vertex and edge numbering:
//
//        ^ y
//        |
//        e1
//   v2 ------ v1
//    |        |
// e2 |        | e4  --> x
//    |        |
//   v3 ------ v4
//        e3

#[derive(Clone, Copy)]
pub enum EdgeNumbers {
	NoEdge,
	Edge1,
	Edge2,
	Edge3,
	Edge4,
}

impl Default for EdgeNumbers {
    fn default() -> Self {
        EdgeNumbers::NoEdge
    }
}

#[derive(Default, Clone, Copy)]
pub struct Feature {
    pub in_edge1: EdgeNumbers,
    pub out_edge1: EdgeNumbers,
    pub in_edge2: EdgeNumbers,
    pub out_edge2: EdgeNumbers,
}

#[derive(Default)]
pub struct Contact {
    pub position: Vec2,
    pub normal: Vec2,
    r1: Vec2,
    r2: Vec2,
    pub separation: f32,
    pn: f32, // accumulated normal impulse
    pt: f32, // accumulated tangent impulse
    pnb: f32, // accumulated normal impulse for position bias
    mass_normal: f32,
    mass_tangent: f32,
    bias: f32,
    pub feature: Feature,
}

pub const MAX_CONTACT_POINT: usize = 2;