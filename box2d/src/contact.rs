use crate::math_utils::Vec2;

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

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Feature {
    pub in_edge1: EdgeNumbers,
    pub out_edge1: EdgeNumbers,
    pub in_edge2: EdgeNumbers,
    pub out_edge2: EdgeNumbers,
}

#[derive(Default, Clone)]
pub struct Contact {
    pub position: Vec2,
    pub normal: Vec2,
    pub r1: Vec2,
    pub r2: Vec2,
    pub separation: f32,
    pub pn: f32,  // accumulated normal impulse
    pub pt: f32,  // accumulated tangent impulse
    pub pnb: f32, // accumulated normal impulse for position bias
    pub mass_normal: f32,
    pub mass_tangent: f32,
    pub bias: f32,
    pub feature: Feature,
}

pub const MAX_CONTACT_POINT: usize = 2;
