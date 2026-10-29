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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EdgeNumbers {
    #[default]
    NoEdge,
    Edge1,
    Edge2,
    Edge3,
    Edge4,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Feature {
    pub in_edge1: EdgeNumbers,
    pub out_edge1: EdgeNumbers,
    pub in_edge2: EdgeNumbers,
    pub out_edge2: EdgeNumbers,
}

#[derive(Default, Clone, Copy)]
pub struct Contact {
    pub position: Vec2,
    pub normal: Vec2,
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

/// Up to `MAX_CONTACT_POINT` contacts of one body pair, without allocation.
#[derive(Default, Clone, Copy)]
pub struct Contacts {
    items: [Contact; MAX_CONTACT_POINT],
    len: usize,
}

impl Contacts {
    pub fn push(&mut self, contact: Contact) {
        self.items[self.len] = contact;
        self.len += 1;
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[Contact] {
        &self.items[..self.len]
    }

    pub fn as_mut_slice(&mut self) -> &mut [Contact] {
        &mut self.items[..self.len]
    }
}
