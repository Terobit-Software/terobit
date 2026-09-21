# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**Keep this file current:** when a change lands that affects anything documented here — commands, crate structure, public API, dependencies, known quirks, project state — update the relevant section as part of that change.

## Current state

Early but working skeleton: the workspace compiles (`cargo check` passes, with a number of dead-code warnings for not-yet-used fields/methods), opens a window, clears it to black, and draws every `(Mesh, Transform)` entity in the current ECS world via the root shader/pipeline, then runs registered frame routines. The bundled test routine spawns a randomly placed/rotated cube (`Mesh::CUBE` + `Transform`) on ~1/256 frames.

Meshes are uploaded to the GPU lazily and drawn instanced. `AssetRegistry` owns the parsed geometry (`assets::MeshData`; `assets::Vertex` is `Pod` so it uploads as-is); `runtime/src/app/mesh.rs`'s `GpuMeshCache` uploads a mesh's vertex/index buffers (`GpuMesh`) the first time an entity referencing that asset id is drawn, and warns once for ids missing from the registry. Each frame `AppState::upload_instances` queries `(&Mesh, &Transform)`, sorts by mesh asset id, writes one `InstanceRaw` model matrix (`Transform::to_matrix`) per entity into a single shared instance buffer (`instance.rs`; doubles from `INITIAL_INSTANCE_CAPACITY` when full, split into 4 `vec4` shader locations since WGSL vertex attributes cap at 4 components), and records a `DrawBatch` (mesh + contiguous instance range) per distinct mesh; the render pass issues one `draw_indexed` per batch. Also bound: a depth buffer (`Depth32Float`, recreated on resize) and a camera uniform bind group holding only the view-projection matrix. There's no real camera yet — `AppState::update_frame` recomputes the view-projection every frame from a fixed placeholder camera (`consts::render::CAMERA_POSITION`/`CAMERA_FOV_Y_RADIANS`/etc.). The fragment shader does simple fixed-light-direction Lambert shading off the vertex normal, rotated by the instance's model matrix (correct for uniform scale only — non-uniform scale would need an inverse-transpose normal matrix).

`Transform::default()` is the identity (unit scale); nothing draws a zero-scale mesh.

Frame pacing: presented frames are paced by vsync in `render()`; frames skipped without presenting (occluded, outdated, or lost surface) sleep for `SKIPPED_FRAME_DELAY` instead, so the `ControlFlow::Poll` + `request_redraw` loop doesn't spin unthrottled while the window is hidden or mid-resize. Note the test routine's 1-in-65536 random terminate still ends the run eventually (~18 min average at 60 fps). The `ERROR winit ... no handler was set` lines sometimes logged at shutdown are harmless: macOS delivers leftover window callbacks after the event loop has exited.

## Commands

```bash
# build everything
cargo build

# type-check quickly
cargo check

# lint
cargo clippy

# run the tests
cargo test -p runtime

# run the client directly
cargo run -p client

# build release, copy assets, and run from out/client (preferred run path)
python3 scripts/run_client.py
```

`scripts/run_client.py` builds `client` in release mode, wipes and recreates `out/client`, copies `client/assets` into it (skipping gracefully if absent — `client/assets/` exists but is currently empty), then runs the binary from `out/client`.

The client writes daily-rotated log files (via `tracing-appender`) to `~/Library/Logs/TerobitClient/` on macOS; the Linux/Windows paths in `get_log_directory()` are placeholders.

Tests live under `runtime/tests/`, one file per `runtime` source module, mirroring `runtime/src/`'s tree (e.g. `runtime/tests/consts/render.rs` tests `runtime/src/consts/render.rs`). Since each top-level file under `runtime/tests/` is compiled as its own separate test crate, a top-level file that needs sibling test files for submodules (`runtime.rs`, `app.rs`, `consts.rs`, `ecs.rs`) pulls them in with `#[path = "..."] mod ...;` pointing into a same-named subdirectory — plain `mod foo;` would look for `foo.rs`/`foo/mod.rs` next to it, not inside that subdirectory. Everything under `runtime/src/app/` and `runtime/src/assets/` is `pub(crate)`, so those modules have nothing an external test crate can name or construct; their test files are placeholders kept for parity with the rest of the tree. CI (`.github/workflows/ci.yml`) runs `cargo clippy --workspace --all-targets --no-deps` then `cargo test -p runtime` on pushes to main and on PRs — `--no-deps` keeps dependencies compiled with plain `rustc` instead of `clippy-driver` so their build artifacts are shared with the `cargo test` step instead of forcing a full rebuild (dependency warnings are already capped to `allow` by Cargo regardless, so this doesn't change what's linted); Dependabot (`.github/dependabot.yml`) groups minor/patch cargo bumps weekly and leaves majors as individual PRs.

## Verifying changes

**Never use screenshots to verify the app** (`screencapture`, browser/simulator screenshots, etc.) unless explicitly instructed to. Verify with other means instead: `cargo test`/`cargo clippy`, and running the client and reading its logs (terminal output and `~/Library/Logs/TerobitClient/`). When a change has no log line to confirm it, add a temporary or permanent `tracing` log (e.g. at `info!`/`debug!`) rather than reaching for a screenshot — but avoid per-frame logging, which floods the terminal.

## Architecture

Cargo workspace with two crates:

- **`runtime`** — the engine library. Public surface: `Runtime` / `RuntimeInitError` (from `runtime/src/runtime.rs`), the `routines` and `handle` modules under it, and `consts`. Everything under `runtime/src/app/` is `pub(crate)`.
- **`client`** — the game binary (`client/src/main.rs`). Sets up a `tracing` subscriber (terminal + rolling file), creates a `Runtime`, registers routines with `add_routine`, and calls `runtime.start()`.

### Runtime flow

`Runtime::start()` (`runtime/src/runtime.rs`) creates the winit `EventLoop`, sets `ControlFlow::Poll`, and drives `RuntimeApp` via `run_app`. Errors surface as `RuntimeInitError` (thiserror).

Inside `runtime/src/app/`:

- `RuntimeApp` (`app.rs`) holds `Option<AppState>`, built lazily in `resumed()` (blocking on the async wgpu setup with `pollster`). Routines registered before then are queued in `pending_routines` and drained into the state on creation. `window_event` handles `CloseRequested` (exit), `Resized` (reconfigures the surface via `AppState::resize`, ignoring zero sizes), and `RedrawRequested` (runs `AppState::run()` and applies any returned `RuntimeCommand`s); `about_to_wait` requests the next redraw to keep the loop going.
- `AppState` (`state.rs`) owns the full wgpu stack (instance, adapter, device, queue, surface, format) plus the `Window`, depth view, `GpuMeshCache`, the instance buffer + per-frame `DrawBatch`es, the camera uniform buffer, and the routine list. `run()` renders a clear + depth-tested pass with one instanced draw per mesh, then runs routines. Surface config values live in `consts`.
- `Window` (`window.rs`) wraps `Arc<winit::window::Window>`.

### Routines

`RuntimeRoutine` (`runtime/src/runtime/routines.rs`) is the per-frame extension point: `priority()` (u8, lower runs earlier; equal priorities run in insertion order), `frame(&self, &RuntimeHandle)`, `name()`, `is_enabled()`. `AppState` keeps routines sorted by priority.

Writes back to the runtime are deferred: each frame, routines get a fresh `RuntimeHandle` (`runtime/src/runtime/handle.rs`) that collects `RuntimeCommand`s in a `RefCell`; the runtime applies them after all routines have run. Currently the only command is `Terminate`.

### Consts

Tunable engine values live in `runtime/src/consts/`: `app.rs` has `WINDOW_TITLE`, `DESIRED_MAXIMUM_FRAME_LATENCY`, `PRESENT_MODE`, `SKIPPED_FRAME_DELAY`; `render.rs` has the placeholder-camera/depth/instancing knobs (`DEPTH_FORMAT`, `CAMERA_FOV_Y_RADIANS`, `CAMERA_NEAR`, `CAMERA_FAR`, `CAMERA_POSITION`, `INITIAL_INSTANCE_CAPACITY`). New tuning knobs should go here rather than inline literals.

### Dependencies

winit 0.30 (with `rwh_06`), wgpu 27, pollster, rand 0.10, bytemuck (with the `derive` feature, for `Pod`/`Zeroable` on `assets::Vertex` and `InstanceRaw`), glam (used for `Mat4`/`Vec3` MVP math — note `Mat4::look_at_rh`/`perspective_rh` are deprecated in this version; use `glam::camera::rh::view::look_at_mat4` and `glam::camera::rh::proj::directx::perspective` instead, which match wgpu's right-handed/Y-up/[0,1]-depth NDC convention), tracing, thiserror 2, tobj 4 (parses OBJ files for `assets::MeshAsset`; loaded with `tobj::GPU_LOAD_OPTIONS` — triangulated, single-indexed — with default features, so the `merging`/`reordering` cargo features and their `LoadOptions` fields aren't available). `iced_wgpu` and `iced_winit` 0.14 are also pulled in (for a planned UI overlay on top of the render stack) but not yet wired into any code — wgpu is pinned to 27 because that's the newest version iced 0.14 supports; bumping wgpu without bumping iced (or vice versa) will break the build. The client additionally uses tracing-appender, tracing-subscriber, and uuid.
