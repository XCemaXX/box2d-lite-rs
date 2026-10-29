use std::ops::{Deref, DerefMut};

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
pub enum Edge {
    E1,
    E2,
    E3,
    E4,
}

/// Edges that produced a contact point. Contacts with the same feature
/// in two steps are the same contact.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Feature {
    pub in_edge1: Option<Edge>,
    pub out_edge1: Option<Edge>,
    pub in_edge2: Option<Edge>,
    pub out_edge2: Option<Edge>,
}

impl Feature {
    /// Swaps the roles of the two boxes.
    pub fn flip(&mut self) {
        (self.in_edge1, self.in_edge2) = (self.in_edge2, self.in_edge1);
        (self.out_edge1, self.out_edge2) = (self.out_edge2, self.out_edge1);
    }
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
}

impl Deref for Contacts {
    type Target = [Contact];

    fn deref(&self) -> &[Contact] {
        &self.items[..self.len]
    }
}

impl DerefMut for Contacts {
    fn deref_mut(&mut self) -> &mut [Contact] {
        &mut self.items[..self.len]
    }
}
