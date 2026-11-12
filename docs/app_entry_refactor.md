# Refactor: app entry (winit + wgpu)

Scope: `gui/` only. `gui_macroquad/`, `box2d/`, `physics/` do not change.
Versions: winit 0.30.13, wgpu 30.0.1. One step = one commit. After each step: `./gui/build.sh`, open in Chrome (WebGPU) and in a browser without WebGPU (WebGL2).

## Problems

| # | Problem | Where |
|---|---|---|
| P1 | The renderer is ready, but nothing wakes the event loop. `try_start` runs only from `window_event`. Start waits for any window event (mouse, resize). | `lib.rs` `try_start`, `window_event` |
| P2 | `ControlFlow::Poll` on the web spins empty iterations. Frames already come from rAF through `request_redraw`. | `lib.rs` `start` |
| P3 | `request_redraw()` after every event. The animation loop works by accident. | `lib.rs` `window_event` |
| P4 | `run_app` on the web throws a JS exception to leave the loop. winit docs recommend `spawn_app`. | `lib.rs` `start` |
| P5 | Three `Option` fields form a hidden state machine. Invalid combinations compile. `window` is stored twice. | `lib.rs` `App` |
| P6 | A panic in `Render::new` drops the sender. `try_recv` returns `Err(Canceled)`, `let ... else return` hides it. Gray screen forever. | `lib.rs` `try_start` |
| P7 | A second `resumed` creates a new window on the same canvas. `Running` keeps the old one. | `lib.rs` `resumed` |
| P8 | Panic hook and logger are set inside `resumed`. An earlier panic has no readable text. | `lib.rs` `resumed` |
| P9 | `last_size` is redundant: on the web `window.inner_size()` is the canvas size. | `lib.rs` `App` |
| P10 | `Render<'a>`: the surface comes from `Arc<Window>` and the font from `include_bytes!`. Both are `'static`. | `render/mod.rs` |
| P11 | `default-features = false, features = ["webgl"]` has no effect: `wgpu_text` depends on `wgpu` with default features, so `webgpu` and `wgsl` are on. `Instance::default()` picks WebGPU when `navigator.gpu` exists, even if no adapter is available (seen on Linux). Then `request_adapter(...).expect` panics, no fallback to WebGL2. | `gui/Cargo.toml`, `render/mod.rs` `init_screen` |
| P12 | WebGL2 limits are requested on a WebGPU adapter too. Compute and storage buffers stay closed. | `render/mod.rs` `init_screen` |

Not a problem: `Pin`, self-referential structs, `create_surface_unsafe`. `Arc<Window>` owns the window; `Surface<'static>` holds the `Arc`. Nothing points into `App`.

## Step 0. Find out the current backend

In `init_screen`, after `request_adapter`:
```rust
log::info!("Backend: {:?}", adapter.get_info().backend);
```
Check the console in Chrome and in Firefox. This tells you what P11 does on your machines today.

Commit: `chore: log the wgpu backend at start`

## Step 1. Panic hook and logger in `start` (P8)

```rust
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    console_log::init().expect("logger");
    ...
}
```
Remove both calls from `resumed`.

Commit: `fix: set panic hook and logger before the event loop`

## Step 2. `Render` without lifetime (P10)

- `pub struct Render { surface: wgpu::Surface<'static>, text_brush: TextBrush<FontRef<'static>>, .. }`
- `Render::new(window: Arc<Window>, ..)`, `init_text() -> TextBrush<FontRef<'static>>`.
- `Receiver<Render>`, `Running { render: Render }`.

Commit: `chore: drop the lifetime parameter of Render`

## Step 3. `Render::new` returns `Result` (P6, P11)

- `init_screen` returns `Result<_, String>` (or `anyhow`). No `expect`.
- Instance: `wgpu::util::new_instance_with_webgpu_detection(&desc).await`. It falls back to GL when WebGPU has no adapter.
- `gui/Cargo.toml`: write the real features:
  `wgpu = { version = "30.0.1", default-features = false, features = ["webgpu", "webgl", "wgsl"] }`.
  Comment: `wgpu_text enables default features anyway`. Or drop `default-features = false`: it does nothing.
- Sender type: `oneshot::Sender<Result<Render, String>>`.

Commit: `fix: fall back to WebGL2 when WebGPU has no adapter`

## Step 4. State enum and wake-up (P1, P5, P7, P9)

```rust
enum State {
    Uninit,
    Initializing { window: Arc<Window>, rx: oneshot::Receiver<Result<Render, String>> },
    Running(Running),
    Failed,
}

pub struct App { proxy: EventLoopProxy<()>, state: State }
```

- `EventLoop::<()>::with_user_event().build()`, `App { proxy: event_loop.create_proxy(), state: State::Uninit }`. `App` loses `Default`.
- `resumed` → `fn create_surfaces(&mut self, el)`. First line: `if !matches!(self.state, State::Uninit) { return; }`.
- The spawned task sends the result, then `proxy.send_event(())`.
- `fn user_event(&mut self, _, (): ())` → `on_wake()`: `try_recv`; `Ok(None)` → return; `Err(Canceled)` or `Err(e)` → log, `State::Failed`; `Ok(render)` → `render.resize(window.inner_size())`, build `Running`, `request_redraw()`.
- Move from `Initializing` with `std::mem::replace(&mut self.state, State::Failed)`.
- Delete `try_start`, `last_size`, `App.window`, `App.pending_render`, `App.running`.
- `window_event`: `match (&mut self.state, event)`; events before `Running` are ignored.

Why `EventLoopProxy<()>` and not `EventLoopProxy<Render>`: winit 0.31 removes `user_event(T)` and keeps only a wake-up without data (`proxy_wake_up`). The data already goes through `oneshot`.

Commit: `fix: wake the event loop when the renderer is ready`

## Step 5. Frame loop (P2, P3, P4)

- Remove `set_control_flow(Poll)`. Default is `Wait`.
- `request_redraw()` only in the `RedrawRequested` branch, after `redraw()`. On the web it maps to `requestAnimationFrame`.
- Input events do not call `request_redraw()`: the rAF loop already runs.
- `event_loop.spawn_app(app)` (`winit::platform::web::EventLoopExtWebSys`) instead of `run_app`.

Check: CPU use in the browser task manager drops when the tab is idle; fps stays at the display rate.

Commit: `fix: rAF-driven redraw, spawn_app on the web`

## Step 6. Limits by capability (P12)

```rust
let base = if adapter.get_downlevel_capabilities().flags.contains(wgpu::DownlevelFlags::COMPUTE_SHADERS) {
    wgpu::Limits::downlevel_defaults()
} else {
    wgpu::Limits::downlevel_webgl2_defaults()
};
let required_limits = base.using_resolution(adapter.limits());
```
box2d does not need compute. Do this step only if box2d is the template for the next project.

Commit: `chore: request limits by adapter capability`

## Optional: native build

- `cfg(target_arch = "wasm32")` around canvas lookup, `spawn_local`, `spawn_app`, `#[wasm_bindgen(start)]`.
- Native: `pollster::block_on` instead of `spawn_local`, `run_app(&mut app)`, `src/main.rs` with `env_logger`.
- winit has `default-features = false`: native needs `x11`/`wayland` features. wgpu default features already bring `vulkan`/`metal`/`dx12`.
- Gain: compiler errors and shader validation errors without the browser; faster loop.

## Migration to winit 0.31 (beta.4 as of 2026-11; not released)

| 0.30 | 0.31 |
|---|---|
| `resumed` | `can_create_surfaces` (required) |
| `user_event(_, ())` | `proxy_wake_up(_)` |
| `proxy.send_event(())` | `proxy.wake_up()` |
| `Arc<Window>` | `Arc<dyn Window>` from `Box<dyn Window>` |
| `spawn_app(app)` | `run_app(app)`; returns at once on the web |

After step 4 the migration is mostly renames.

## References
- winit `EventLoopExtWebSys::spawn_app`: https://docs.rs/winit/0.30.13/winit/platform/web/trait.EventLoopExtWebSys.html
- winit `PollStrategy`: https://docs.rs/winit/0.30.13/winit/platform/web/enum.PollStrategy.html
- wgpu examples framework: https://github.com/gfx-rs/wgpu/blob/trunk/examples/features/src/framework.rs
- `new_instance_with_webgpu_detection`: https://docs.rs/wgpu/30.0.1/wgpu/util/fn.new_instance_with_webgpu_detection.html
- winit 0.31 `ApplicationHandler`: https://docs.rs/winit/0.31.0-beta.4/winit/application/trait.ApplicationHandler.html
