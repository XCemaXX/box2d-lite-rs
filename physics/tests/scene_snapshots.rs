//! Trajectories of all demo scenes, compared with stored snapshots.
//! A change in the solver shows up as a diff, down to the last bit of a float.
//! Update after an intended change: `cargo insta review` or `INSTA_UPDATE=always cargo test`.

use std::fmt::Write;

use physics::PhysicsState;

const DT: f32 = 1.0 / 60.0;
const STEPS: usize = 240;
const SAMPLE_EVERY: usize = 30;

fn trace(scene: usize) -> String {
    // Fixed jitter: scenes with random placement are reproducible too.
    let mut jitter = [0.9, -0.4, 0.1, -1.0, 0.6, -0.7].into_iter().cycle();
    let mut state = PhysicsState::with_jitter(move || jitter.next().unwrap());
    state.set_scene(scene).unwrap();
    let mut out = format!("{state}\n");
    for step in 0..=STEPS {
        if step % SAMPLE_EVERY == 0 {
            let contacts = state.collide_points().count();
            writeln!(out, "step {step}, contacts {contacts}").unwrap();
            for (i, r) in state.rectangles().enumerate() {
                // `{:?}` prints the shortest text that parses back to the same f32.
                let c = r.center;
                writeln!(out, "  {i}: {:?} {:?} {:?}", c.x, c.y, r.rotation).unwrap();
            }
        }
        state.step(DT);
    }
    out
}

#[test]
fn scenes() {
    for scene in 0..PhysicsState::scene_count() {
        insta::assert_snapshot!(format!("scene_{scene}"), trace(scene));
    }
}
