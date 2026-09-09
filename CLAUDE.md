# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**Keep this file current:** when a change lands that affects anything documented here — commands, crate structure, public API, dependencies, known quirks, project state — update the relevant section as part of that change.

## Current state

Early but working skeleton: the workspace compiles (`cargo check` passes, with a number of dead-code warnings for not-yet-used fields/methods), opens a window, clears it to a random red/blue color each frame, and runs registered frame routines. The bundled test routine prints a random `u16` every frame and terminates the runtime when it hits 0.

Known quirk: frame pacing comes entirely from vsync in `render()`. When the window is occluded (covered/minimized), `get_current_texture()` returns early without presenting, so the `ControlFlow::Poll` + `request_redraw` loop spins unthrottled at 100% CPU — which is also why the test routine's 1-in-65536 terminate fires almost instantly once the window is hidden.

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

There are no tests yet.

## Architecture

Cargo workspace with two crates:

- **`runtime`** — the engine library. Public surface: `Runtime` / `RuntimeInitError` (from `runtime/src/runtime.rs`), the `routines` and `handle` modules under it, and `consts`. Everything under `runtime/src/app/` is `pub(crate)`.
- **`client`** — the game binary (`client/src/main.rs`). Sets up a `tracing` subscriber (terminal + rolling file), creates a `Runtime`, registers routines with `add_routine`, and calls `runtime.start()`.

### Runtime flow

`Runtime::start()` (`runtime/src/runtime.rs`) creates the winit `EventLoop`, sets `ControlFlow::Poll`, and drives `RuntimeApp` via `run_app`. Errors surface as `RuntimeInitError` (thiserror).

Inside `runtime/src/app/`:

- `RuntimeApp` (`app.rs`) holds `Option<AppState>`, built lazily in `resumed()` (blocking on the async wgpu setup with `pollster`). Routines registered before then are queued in `pending_routines` and drained into the state on creation. `window_event` handles `CloseRequested` (exit) and `RedrawRequested` (runs `AppState::run()` and applies any returned `RuntimeCommand`s); `about_to_wait` requests the next redraw to keep the loop going.
- `AppState` (`state.rs`) owns the full wgpu stack (instance, adapter, device, queue, surface, format) plus the `Window` and the routine list. `run()` renders a clear pass, then runs routines. Surface config values live in `consts`.
- `Window` (`window.rs`) wraps `Arc<winit::window::Window>`.

### Routines

`RuntimeRoutine` (`runtime/src/runtime/routines.rs`) is the per-frame extension point: `priority()` (u8, lower runs earlier; equal priorities run in insertion order), `frame(&self, &RuntimeHandle)`, `name()`, `is_enabled()`. `AppState` keeps routines sorted by priority.

Writes back to the runtime are deferred: each frame, routines get a fresh `RuntimeHandle` (`runtime/src/runtime/handle.rs`) that collects `RuntimeCommand`s in a `RefCell`; the runtime applies them after all routines have run. Currently the only command is `Terminate`.

### Consts

Tunable engine values live in `runtime/src/consts/` (`app_consts.rs`): `WINDOW_TITLE`, `DESIRED_MAXIMUM_FRAME_LATENCY`, `PRESENT_MODE`. New tuning knobs should go here rather than inline literals.

### Dependencies

winit 0.30 (with `rwh_06`), wgpu 30, pollster, rand 0.10, tracing, thiserror 2. The client additionally uses tracing-appender, tracing-subscriber, and uuid.
