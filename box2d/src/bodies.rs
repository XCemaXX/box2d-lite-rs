use crate::body::Body;

/// Handle of a body in a `World`. After the body is removed the handle stays invalid,
/// even when its slot holds a new body. Handles of another `World` are not detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BodyId {
    // Field order matters: `Ord` sorts by index first, the order bodies were added.
    index: u32,
    generation: u32,
}

struct Slot {
    generation: u32,
    body: Option<Body>,
}

/// Generational storage: a removed body frees its slot for reuse.
#[derive(Default)]
pub(crate) struct Bodies {
    slots: Vec<Slot>,
    free: Vec<u32>,
}

impl Bodies {
    pub fn insert(&mut self, body: Body) -> BodyId {
        if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index as usize];
            slot.body = Some(body);
            return BodyId {
                index,
                generation: slot.generation,
            };
        }
        let index = u32::try_from(self.slots.len()).expect("too many bodies");
        self.slots.push(Slot {
            generation: 0,
            body: Some(body),
        });
        BodyId {
            index,
            generation: 0,
        }
    }

    pub fn remove(&mut self, id: BodyId) -> Option<Body> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation != id.generation {
            return None;
        }
        let body = slot.body.take()?;
        slot.generation = slot.generation.wrapping_add(1);
        self.free.push(id.index);
        Some(body)
    }

    pub fn clear(&mut self) {
        for (index, slot) in self.slots.iter_mut().enumerate() {
            if slot.body.take().is_some() {
                slot.generation = slot.generation.wrapping_add(1);
                self.free.push(index as u32);
            }
        }
    }

    pub fn get(&self, id: BodyId) -> Option<&Body> {
        let slot = self.slots.get(id.index as usize)?;
        (slot.generation == id.generation)
            .then_some(slot.body.as_ref())
            .flatten()
    }

    pub fn get_mut(&mut self, id: BodyId) -> Option<&mut Body> {
        let slot = self.slots.get_mut(id.index as usize)?;
        (slot.generation == id.generation)
            .then_some(slot.body.as_mut())
            .flatten()
    }

    /// Two different live bodies at once.
    /// Panics on a stale id: arbiters and joints are removed together with their bodies.
    pub fn pair_mut(&mut self, a: BodyId, b: BodyId) -> (&mut Body, &mut Body) {
        let [slot_a, slot_b] = self
            .slots
            .get_disjoint_mut([a.index as usize, b.index as usize])
            .expect("two different bodies");
        (live(slot_a, a), live(slot_b, b))
    }

    /// Live bodies in the order of their slots.
    pub fn iter(&self) -> impl Iterator<Item = (BodyId, &Body)> {
        self.slots.iter().enumerate().filter_map(|(index, slot)| {
            let id = BodyId {
                index: index as u32,
                generation: slot.generation,
            };
            slot.body.as_ref().map(|body| (id, body))
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Body> {
        self.slots.iter_mut().filter_map(|slot| slot.body.as_mut())
    }

    pub fn len(&self) -> usize {
        self.slots.len() - self.free.len()
    }
}

fn live(slot: &mut Slot, id: BodyId) -> &mut Body {
    assert_eq!(slot.generation, id.generation, "stale body id");
    slot.body.as_mut().expect("stale body id")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math_utils::Vec2;

    fn body() -> Body {
        Body::new_static(Vec2::new(1.0, 1.0), Vec2::ZERO)
    }

    #[test]
    fn removed_id_is_invalid_after_slot_reuse() {
        let mut bodies = Bodies::default();
        let old = bodies.insert(body());
        assert!(bodies.remove(old).is_some());
        let new = bodies.insert(body());
        assert_eq!(old.index, new.index);
        assert!(bodies.get(old).is_none());
        assert!(bodies.get(new).is_some());
        assert!(bodies.remove(old).is_none());
    }

    #[test]
    fn clear_invalidates_ids() {
        let mut bodies = Bodies::default();
        let id = bodies.insert(body());
        bodies.clear();
        assert_eq!(bodies.len(), 0);
        let reused = bodies.insert(body());
        assert!(bodies.get(id).is_none());
        assert!(bodies.get(reused).is_some());
    }

    #[test]
    #[should_panic(expected = "two different bodies")]
    fn pair_of_same_body_panics() {
        let mut bodies = Bodies::default();
        let id = bodies.insert(body());
        bodies.pair_mut(id, id);
    }
}
