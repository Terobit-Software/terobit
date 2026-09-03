# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Current state

The repo was recently reset by a full rewrite (PR #1, `major/full-rewrite`); it is an early skeleton, not a working engine yet. The most recent commit is a quicksave and HEAD is known not to compile (e.g. `AppState::new` is async but never called or awaited, `RuntimeApp` never constructs its `AppState`, and there are unused imports). Verify with `cargo check` before assuming a clean baseline.

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

`scripts/run_client.py` builds `client` in release mode, wipes and recreates `out/client`, copies `client/assets` into it, then runs the binary from there. Note `client/assets/` does not exist yet, so the script currently fails at the copy step until that directory is created.

There are no tests yet.

## Architecture

Cargo workspace with two crates:

- **`runtime`** — the engine library. Public surface is just `Runtime` (re-exported from `runtime/src/runtime.rs`); everything under `runtime/src/app/` is `pub(crate)`.
- **`client`** — the game binary (`client/src/main.rs`). Sets up a `tracing` subscriber, creates a `Runtime`, and calls `runtime.start()`.

### Runtime flow

`Runtime::start()` (`runtime/src/runtime.rs`) creates the winit `EventLoop`, sets `ControlFlow::Poll`, constructs a `RuntimeApp`, and drives it from the event loop closure. Errors surface as `RuntimeInitError` (thiserror).

Inside `runtime/src/app/`:

- `RuntimeApp` (`app.rs`) holds `Option<AppState>`, which stays `None` until `AppState` can be built with a real event loop (currently never, this is part of the unfinished wiring).
- `AppState` (`state.rs`) owns the `wgpu::Instance` and the `Window`. The instance is created without a display handle on purpose, winit 0.29 can't hand out an owned one and wgpu only needs it for gles/wayland.
- `Window` (`window.rs`) wraps `Arc<winit::window::Window>`.

### Dependencies

winit 0.29 (with `rwh_06`), wgpu 30, pollster (for blocking on the async wgpu setup), tracing, thiserror. The `[target.aarch64-apple-darwin] rustflags` blocks in the crate `Cargo.toml`s are not valid package manifest keys (that config belongs in `.cargo/config.toml`) and cargo ignores them.
