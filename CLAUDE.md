# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**Keep this file current:** when a change lands that affects anything documented here — commands, crate structure, public API, dependencies, known quirks, project state — update the relevant section as part of that change.

## Current state

Early but working skeleton: the workspace compiles (`cargo check` passes, with a number of dead-code warnings for not-yet-used fields/methods), opens a window, clears it to black, and draws a spinning red-shaded cube via the root shader/pipeline (embedded with `include_str!` from `runtime/src/app/shaders/root.wgsl`), then runs registered frame routines. The bundled test routine rolls a random `u16` every frame and terminates the runtime when it hits 0.

The cube (`runtime/src/app/mesh.rs`, `CUBE_VERTICES`/`CUBE_INDICES`) is drawn instanced: a shared vertex/index buffer plus a per-instance vertex buffer (`runtime/src/app/instance.rs`, `InstanceRaw`) carrying one model matrix per instance (split into 4 `vec4` shader locations since WGSL vertex attributes cap at 4 components), bound alongside a depth buffer (`Depth32Float`, recreated on resize) and a camera uniform bind group holding only the view-projection matrix. There's no real camera, scene graph, or multiple instances yet — `AppState::update_frame` recomputes the view-projection every frame from a fixed placeholder camera (`consts::render::CAMERA_POSITION`/`CAMERA_FOV_Y_RADIANS`/etc.) and writes a single instance whose model matrix spins over time (`CUBE_SPIN_RADIANS_PER_SECOND`), just so the shape reads as 3D before "move the camera" and "multiple cubes" are tackled. The fragment shader does simple fixed-light-direction Lambert shading off the per-face vertex normal, rotated into world space by the instance's model matrix.

Frame pacing: presented frames are paced by vsync in `render()`; frames skipped without presenting (occluded, outdated, or lost surface) sleep for `SKIPPED_FRAME_DELAY` instead, so the `ControlFlow::Poll` + `request_redraw` loop doesn't spin unthrottled while the window is hidden or mid-resize. Note the test routine's 1-in-65536 random terminate still ends the run eventually (~18 min average at 60 fps). The `ERROR winit ... no handler was set` lines sometimes logged at shutdown are harmless: macOS delivers leftover window callbacks after the event loop has exited.

## Commands

```bash
# build everything
cargo build

# type-check quickly
cargo check

# lint
cargo clippy

# run the client directly
cargo run -p client

# build release, copy assets, and run from out/client (preferred run path)
python3 scripts/run_client.py
```

`scripts/run_client.py` builds `client` in release mode, wipes and recreates `out/client`, copies `client/assets` into it (skipping gracefully if absent — `client/assets/` exists but is currently empty), then runs the binary from `out/client`.

The client writes daily-rotated log files (via `tracing-appender`) to `~/Library/Logs/TerobitClient/` on macOS; the Linux/Windows paths in `get_log_directory()` are placeholders.

There are no tests yet. CI (`.github/workflows/ci.yml`) runs `cargo clippy --workspace --all-targets` on pushes to main and on PRs; Dependabot (`.github/dependabot.yml`) groups minor/patch cargo bumps weekly and leaves majors as individual PRs.

## Architecture

Cargo workspace with two crates:

- **`runtime`** — the engine library. Public surface: `Runtime` / `RuntimeInitError` (from `runtime/src/runtime.rs`), the `routines` and `handle` modules under it, and `consts`. Everything under `runtime/src/app/` is `pub(crate)`.
- **`client`** — the game binary (`client/src/main.rs`). Sets up a `tracing` subscriber (terminal + rolling file), creates a `Runtime`, registers routines with `add_routine`, and calls `runtime.start()`.

### Runtime flow

`Runtime::start()` (`runtime/src/runtime.rs`) creates the winit `EventLoop`, sets `ControlFlow::Poll`, and drives `RuntimeApp` via `run_app`. Errors surface as `RuntimeInitError` (thiserror).

Inside `runtime/src/app/`:

- `RuntimeApp` (`app.rs`) holds `Option<AppState>`, built lazily in `resumed()` (blocking on the async wgpu setup with `pollster`). Routines registered before then are queued in `pending_routines` and drained into the state on creation. `window_event` handles `CloseRequested` (exit), `Resized` (reconfigures the surface via `AppState::resize`, ignoring zero sizes), and `RedrawRequested` (runs `AppState::run()` and applies any returned `RuntimeCommand`s); `about_to_wait` requests the next redraw to keep the loop going.
- `AppState` (`state.rs`) owns the full wgpu stack (instance, adapter, device, queue, surface, format) plus the `Window`, depth view, cube vertex/index buffers, the instance buffer, the camera uniform buffer, and the routine list. `run()` renders a clear + depth-tested, instanced cube pass, then runs routines. Surface config values live in `consts`.
- `Window` (`window.rs`) wraps `Arc<winit::window::Window>`.

### Routines

`RuntimeRoutine` (`runtime/src/runtime/routines.rs`) is the per-frame extension point: `priority()` (u8, lower runs earlier; equal priorities run in insertion order), `frame(&self, &RuntimeHandle)`, `name()`, `is_enabled()`. `AppState` keeps routines sorted by priority.

Writes back to the runtime are deferred: each frame, routines get a fresh `RuntimeHandle` (`runtime/src/runtime/handle.rs`) that collects `RuntimeCommand`s in a `RefCell`; the runtime applies them after all routines have run. Currently the only command is `Terminate`.

### Consts

Tunable engine values live in `runtime/src/consts/`: `app.rs` has `WINDOW_TITLE`, `DESIRED_MAXIMUM_FRAME_LATENCY`, `PRESENT_MODE`, `SKIPPED_FRAME_DELAY`; `render.rs` has the placeholder-camera/depth knobs (`DEPTH_FORMAT`, `CAMERA_FOV_Y_RADIANS`, `CAMERA_NEAR`, `CAMERA_FAR`, `CAMERA_POSITION`, `CUBE_SPIN_RADIANS_PER_SECOND`). New tuning knobs should go here rather than inline literals.

### Dependencies

winit 0.30 (with `rwh_06`), wgpu 27, pollster, rand 0.10, bytemuck (with the `derive` feature, for `Pod`/`Zeroable` on `mesh::Vertex`), glam (used for `Mat4`/`Vec3` MVP math — note `Mat4::look_at_rh`/`perspective_rh` are deprecated in this version; use `glam::camera::rh::view::look_at_mat4` and `glam::camera::rh::proj::directx::perspective` instead, which match wgpu's right-handed/Y-up/[0,1]-depth NDC convention), tracing, thiserror 2. `iced_wgpu` and `iced_winit` 0.14 are also pulled in (for a planned UI overlay on top of the render stack) but not yet wired into any code — wgpu is pinned to 27 because that's the newest version iced 0.14 supports; bumping wgpu without bumping iced (or vice versa) will break the build. The client additionally uses tracing-appender, tracing-subscriber, and uuid.
