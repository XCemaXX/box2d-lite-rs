mod demo_scenes;
pub mod primitives;

use std::fmt;

use box2d::{Body, BodyId, Joint, Vec2, World};
use primitives::{Line, Point, Rectangle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownScene(pub usize);

impl fmt::Display for UnknownScene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "no demo scene with index {}", self.0)
    }
}

impl std::error::Error for UnknownScene {}

pub struct PhysicsState {
    world: World,
    current_scene: usize,
    jitter: Box<dyn FnMut() -> f32>,
}

const SCALE_MULT: f32 = 100.0;
const GRAVITY: f32 = -500.0;
// The solver is stable only for small steps.
const MAX_SUBSTEP_DT: f32 = 1.0 / 60.0;
// After a long pause (hidden tab, dragged window) drop the lost time, don't simulate it.
const MAX_SUBSTEPS: u32 = 5;

impl PhysicsState {
    fn add_unmovable_body(&mut self, w: f32, h: f32, pos_x: f32, pos_y: f32) -> BodyId {
        let body = Body::new_static(to_world(w, h), to_world(pos_x, pos_y));
        self.world.add_body(body)
    }

    fn add_body(&mut self, w: f32, h: f32, mass: f32, pos_x: f32, pos_y: f32) -> BodyId {
        let body = Body::new_dynamic(to_world(w, h), mass, to_world(pos_x, pos_y));
        self.world.add_body(body)
    }

    fn add_joint(
        &mut self,
        body1: BodyId,
        body2: BodyId,
        anchor_x: f32,
        anchor_y: f32,
    ) -> &mut Joint {
        self.world
            .add_joint(body1, body2, to_world(anchor_x, anchor_y))
            .expect("scene joint links two different live bodies")
    }

    /// Scene 0 without randomness: the same every run.
    pub fn new() -> Self {
        Self::with_jitter(|| 0.0)
    }

    /// Scene 0. `jitter` returns a value in `[-1, 1]`; some scenes use it to place
    /// bodies. It is kept for restarts and scene changes.
    pub fn with_jitter(jitter: impl FnMut() -> f32 + 'static) -> Self {
        let mut state = PhysicsState {
            world: new_world(),
            current_scene: 0,
            jitter: Box::new(jitter),
        };
        state.load_scene(0);
        state
    }

    /// On error the current scene stays.
    pub fn set_scene(&mut self, scene: usize) -> Result<(), UnknownScene> {
        let init = demo_scenes::scene(scene).ok_or(UnknownScene(scene))?.init;
        self.world = new_world();
        self.current_scene = scene;
        init(self);
        Ok(())
    }

    /// For indices that are known to be valid.
    fn load_scene(&mut self, scene: usize) {
        self.set_scene(scene).expect("scene index in range");
    }

    fn jitter(&mut self) -> f32 {
        (self.jitter)().clamp(-1.0, 1.0)
    }

    pub fn scene_count() -> usize {
        demo_scenes::scene_count()
    }

    /// Advances the world by `dt`, split into equal substeps.
    pub fn step(&mut self, dt: f32) {
        let (n, h) = substeps(dt);
        for _ in 0..n {
            self.world.step(h);
        }
    }

    pub fn rectangles(&self) -> impl Iterator<Item = Rectangle> + '_ {
        self.world.bodies().map(|body| Rectangle {
            center: to_point(body.position),
            width: body.size.x / SCALE_MULT,
            height: body.size.y / SCALE_MULT,
            rotation: body.rotation.angle(),
        })
    }

    pub fn collide_points(&self) -> impl Iterator<Item = Point> + '_ {
        self.world.collide_points().map(to_point)
    }

    pub fn joint_lines(&self) -> impl Iterator<Item = Line> + '_ {
        self.world.joint_lines().map(|(start, end)| Line {
            p1: to_point(start),
            p2: to_point(end),
        })
    }

    pub fn add_rectangle(&mut self, x: f32, y: f32) {
        self.add_body(0.15, 0.15, 200.0, x, y);
    }

    pub fn restart(&mut self) {
        self.load_scene(self.current_scene);
    }

    pub fn change_to_next_scene(&mut self) {
        self.load_scene((self.current_scene + 1) % demo_scenes::scene_count());
    }

    pub fn change_to_prev_scene(&mut self) {
        let count = demo_scenes::scene_count();
        self.load_scene((self.current_scene + count - 1) % count);
    }
}

impl Default for PhysicsState {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PhysicsState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Scene {}: {}. Bodies: {}",
            self.current_scene + 1,
            demo_scenes::scene(self.current_scene).map_or("?", |s| s.name),
            self.world.body_count()
        )
    }
}

fn new_world() -> World {
    const ITERATIONS: u32 = 10;
    World::new(Vec2::new(0.0, GRAVITY), ITERATIONS)
}

fn to_world(x: f32, y: f32) -> Vec2 {
    Vec2::new(x * SCALE_MULT, y * SCALE_MULT)
}

fn to_point(v: Vec2) -> Point {
    Point {
        x: v.x / SCALE_MULT,
        y: v.y / SCALE_MULT,
    }
}

/// Returns the count and the size of substeps for a frame of `dt`.
/// Substeps are equal: a tiny last substep makes the contact bias explode.
fn substeps(dt: f32) -> (u32, f32) {
    if dt <= 0.0 {
        return (0, 0.0);
    }
    let dt = dt.min(MAX_SUBSTEP_DT * MAX_SUBSTEPS as f32);
    let n = (dt / MAX_SUBSTEP_DT).ceil() as u32;
    (n, dt / n as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_120hz_is_one_substep() {
        assert_eq!(substeps(1.0 / 120.0), (1, 1.0 / 120.0));
    }

    #[test]
    fn substeps_are_equal_and_small() {
        let (n, h) = substeps(0.02);
        assert_eq!(n, 2);
        assert!((h - 0.01).abs() < 1e-6);
    }

    #[test]
    fn long_pause_is_dropped() {
        assert_eq!(substeps(10.0), (MAX_SUBSTEPS, MAX_SUBSTEP_DT));
    }

    // Soft joints must not depend on the display refresh rate.
    #[test]
    fn bridge_sag_does_not_depend_on_step() {
        const BRIDGE: usize = 6;
        let sag = |hz: f32| {
            let mut state = PhysicsState::new();
            state.set_scene(BRIDGE).unwrap();
            for _ in 0..(hz as usize * 4) {
                state.step(1.0 / hz);
            }
            let n = state.rectangles().count();
            state.rectangles().nth(n / 2).unwrap().center.y
        };
        assert!((sag(60.0) - sag(240.0)).abs() < 0.01);
    }

    #[test]
    fn unknown_scene_keeps_current() {
        let mut state = PhysicsState::new();
        state.set_scene(2).unwrap();
        let count = PhysicsState::scene_count();
        assert_eq!(state.set_scene(count), Err(UnknownScene(count)));
        assert_eq!(state.current_scene, 2);
    }

    #[test]
    fn no_time_no_steps() {
        assert_eq!(substeps(0.0).0, 0);
    }
}
