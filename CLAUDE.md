# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Guidelines

You, the agent, should NEVER under ANY circumstances edit or interact with:

- Authentication code, such as, but not limited to, login, JWT, sessions, or passwords.
- Validation code, referring to anything that takes currently unsanitized input from the user, other users, the server, or external files.
- Critical logic, such as, but not limited to, trading of items handling or sorting users into groups for safety and security reasons.
- Anything handling personal or sensitive data from the user, the environment, or the instances.
- Anything security or performance critical.

## Core Philosophy & Safety

- **Memory Management:** Be explicit about ownership. Always use `&T` or `&mut T` for borrows. Explicitly call `.clone()` on non-`Copy` types and do not hide hidden clones in closures or loops. Document usage of `Arc` or `Rc`.
- **Unsafe Rust:** NEVER use `unsafe` unless absolutely necessary. If used, you MUST document the safety invariants and assumptions.
- **Error Handling:** Prefer `anyhow` for application-level errors and `thiserror` for library crates. Do not use unwrap() or expect() in production code without explicit, documented rationale (e.g., during initialization/bootstrapping).

## Rust Coding Standards

- **Pattern Matching:** Use pattern matching exhaustively. Avoid catch-all `_` patterns whenever possible unless handling genuinely unrecoverable states.
- **Formatting:** Use the `format!` macro for string formatting over manual concatenation.
- **Idiomatic Loops:** Prefer Rust’s robust standard library iterators and iterator adapters (e.g., `map`, `filter`, `fold`) over manual `for` loops. Use `.enumerate()` instead of manual counter variables.
- **Conditionals:** Prefer `if let` and `while let` for single-pattern matching rather than full `match` blocks.
- **Documentation:** Public APIs, structs, and traits MUST be documented with standard rustdoc (`///`).

## Cargo & Build Guidelines

- **Dependencies:** Keep `Cargo.toml` clean. Evaluate the necessity of heavy dependencies (e.g., `tokio` vs `async-std`).
- **Workspaces:** For multi-crate projects, maintain strict crate isolation in `Cargo.toml`.
- **Testing:** Integration tests should go in the `tests/` directory. Unit tests should be written inline in the corresponding module under a `#[cfg(test)]` block.
- **Linting & CI:** All code changes must compile cleanly and pass `cargo clippy` without warnings.

Code that may be considered confusing or hard to understand should be documented with comments.
Comments should be simple and not complex. Casual language (such as mostly lowercase), no emojis or symbols, and maybe a little humor too. For example:
Good (do this):

```rust
// creates 5
let v: i32 = 5
```

Bad (dont do this):

```rust
// ❗ Creates a variable named 'v' with an explicit type of i32 and an initialized value of 5.
let v: i32 = 5
```

## Commands

```bash
# Build all crates
cargo build

# Run the client (the only binary)
cargo run -p client

# Run tests
cargo test

# Run tests for a specific crate
cargo test -p runtime
cargo test -p client

# Lint
cargo clippy

# Release build (uses single codegen-unit for max optimization)
cargo build --release
cargo run -p client --release
```

## Architecture

This is a Cargo workspace with two crates:

- **`runtime`** — the engine library. Exposes `Runtime`, `RuntimeContext`, and ECS components.
- **`client`** — the game binary. Depends on `runtime` and defines the actual scene/game logic.

### Runtime lifecycle

`Runtime` (in `runtime/src/core/runtime.rs`) is the public entry point. Users register callbacks then call `runtime.start()`:

- `on_start(|ctx| ...)` — fires once after the window and GPU are ready
- `on_frame(|ctx, delta| ...)` — fires every frame before the render call
- `on_key(|ctx, key_input| ...)` — fires on keyboard events

Internally, `Runtime` owns an `App` (the winit `ApplicationHandler`) and an `EventLoop`. `App` holds `AppState` (GPU state + scenes) and an `EventHandlerContainer` (the registered callbacks).

### Rendering pipeline

`AppState` (`runtime/src/core/app/app_state.rs`) owns all wgpu objects. Each frame:

1. `on_frame` callback runs (user updates transforms)
2. `upload_camera()` — queries the scene for the first `(Transform, Camera)` entity and writes its position to the camera uniform buffer
3. `accumulate_meshes()` — groups all `(Transform, Mesh)` entities by mesh ID and builds `InstanceRaw` batches
4. Instance data is uploaded to a single pre-allocated `instance_buffer` (capacity: 500,000 instances)
5. One draw call per unique mesh ID via `draw_indexed` + instancing

The single WGSL shader is at `runtime/src/core/app/shaders/root.wgsl`. It takes a `view_proj` uniform (camera) and a per-instance model matrix (4×vec4 at locations 3–6).

### ECS

Scenes are `bevy_ecs::world::World` instances stored in a `HashMap<String, World>` on `AppState`. The first scene created becomes the active rendering scene (`current_scene`).

`SceneHandle` (`runtime/src/core/ecs/scene_handle.rs`) is a thin wrapper around `&mut World` returned by `RuntimeContext::create_scene` / `get_scene`. Use `scene.query::<(...)>()` and `.par_iter_mut()` (via rayon) for parallel transform updates.

**ECS components** (all in `runtime/src/core/ecs/components/`):
- `Transform` — position (`Vec3`), rotation (`Quat`), scale (`Vec3`); converts to `Mat4` via `to_mat4()`
- `Mesh` — references a `MeshSourceId`; construct with `Mesh::cube()` or `Mesh::from_asset(...)`
- `Camera` — marker component; the renderer uses the first entity with `(Transform, Camera)` as the camera

### Mesh system

`MeshSourceId` is a `usize` newtype. Built-in meshes are registered at startup by `MeshSourceHandler::init_default_meshes`. Currently only `MeshSourceId::CUBE` exists. GPU mesh data (vertex + index buffers) lives in `AppState::mesh_registry`.

### Key constants (`runtime/src/consts.rs`)

- `MSAA_SAMPLE_COUNT = 1` — MSAA is currently disabled
- `DEFAULT_FOV = 45.0` degrees
- Camera far plane in `RenderCamera` is hardcoded to `10000.0` (overrides the consts file values)
