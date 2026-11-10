use crate::layout::BUTTONS;
use physics::PhysicsState;

const FPS_PERIOD: f32 = 0.4;
const CONTROLS: &str = "Controls: 1-9 scenes; Space: restart; P, N - prev, next scene;
               Click - add box";

#[derive(Clone, Copy)]
pub enum Event {
    Restart,
    CreateBox(f32, f32),
    RunScene(usize),
    NextScene,
    PrevScene,
}

/// The sandbox without the platform: input in world coordinates, a step, the HUD text.
pub struct Game {
    physics: PhysicsState,
    // `None` outside the field.
    cursor: Option<[f32; 2]>,
    fps: FpsCounter,
}

impl Game {
    pub fn new() -> Self {
        Self {
            physics: PhysicsState::with_jitter(|| rand::random_range(-1.0..=1.0)),
            cursor: None,
            fps: FpsCounter::default(),
        }
    }

    /// `pos` is in world coordinates, `None` outside the field.
    pub fn pointer_moved(&mut self, pos: Option<[f32; 2]>) {
        self.cursor = pos;
    }

    /// A press on the bars does nothing.
    pub fn pointer_pressed(&mut self) {
        let Some([x, y]) = self.cursor else {
            return;
        };
        let event = BUTTONS
            .iter()
            .find(|b| b.rect.contains(x, y))
            .map_or(Event::CreateBox(x, y), |b| b.event);
        self.apply(event);
    }

    pub fn apply(&mut self, event: Event) {
        let physics = &mut self.physics;
        match event {
            Event::Restart => physics.restart(),
            Event::CreateBox(x, y) => physics.add_rectangle(x, y),
            Event::RunScene(scene) => {
                if let Err(e) = physics.set_scene(scene) {
                    log::warn!("{e}");
                }
            }
            Event::NextScene => physics.change_to_next_scene(),
            Event::PrevScene => physics.change_to_prev_scene(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.fps.add_frame(dt);
        self.physics.step(dt);
    }

    pub fn physics(&self) -> &PhysicsState {
        &self.physics
    }

    pub fn hud(&self) -> String {
        let (fps, physics) = (self.fps.value, &self.physics);
        let text = format!("fps: {fps:.1}\n{physics}\n{CONTROLS}");
        if !cfg!(debug_assertions) {
            return text;
        }
        match self.cursor {
            Some([x, y]) => format!("Mouse: {x:.3} {y:.3}\n{text}"),
            None => format!("Mouse: outside\n{text}"),
        }
    }
}

/// Average fps, updated once per `FPS_PERIOD` so the number is readable.
#[derive(Default)]
struct FpsCounter {
    value: f32,
    frames: u32,
    time: f32,
}

impl FpsCounter {
    fn add_frame(&mut self, dt: f32) {
        self.frames += 1;
        self.time += dt;
        if self.time >= FPS_PERIOD {
            self.value = self.frames as f32 / self.time;
            self.frames = 0;
            self.time = 0.0;
        }
    }
}
