# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Guidelines

You, the agent, should NEVER under ANY circumstances edit or interact with:

- Authentication code, such as, but not limited to, login, JWT, sessions, or passwords.
- Validation code, referring to anything that takes currently unsanitized input from the user, other users, the server, or external files.
- Critical logic, such as, but not limited to, trading of items handling or sorting users into groups for safety and security reasons.
- Anything handling personal or sensitive data from the user, the environment, or the instances.
- Anything security or performance critical that isn't boilerplate or generic code.

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

You should almost never add comments, except sometimes on non-trivial code, function definitions if the arguments aren't self-explanatory, and class definitions and their members.
Comments should be simple and not complex. Casual language (such as mostly lowercase), no emojis or symbols, and maybe a little humor too. Never use em-dashes (—), instead use commas. For example:
Good (do this):

```rust
// creates 5
let v: i32 = 5
```

^^^ in this situation you would not need a comment, this is just an example of a comment

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
- **`client`** — the game binary (bin name `terobit`, `client/src/main.rs`). Depends on `runtime` and defines the actual scene/game logic.

### Runtime lifecycle

`Runtime` (in `runtime/src/core/runtime.rs`) is the public entry point. Users register callbacks then call `runtime.start()`:

- `on_start(|ctx| ...)` — fires once after the window and GPU are ready
- `on_frame(|ctx, delta| ...)` — fires every frame before physics steps and the render call
- `on_key(|ctx, key_input| ...)` — fires on keyboard events
- `on_mouse_motion(|ctx, dx, dy| ...)` — fires on raw `DeviceEvent::MouseMotion` (physical pixel deltas, ideal for fps look)

Internally, `Runtime` owns an `App` (the winit `ApplicationHandler`) and an `EventLoop`. `App` (`runtime/src/core/app/app.rs`) holds `AppState` (GPU state + scenes) and an `EventHandlerContainer` (the registered callbacks). Each `RedrawRequested`, `App` runs `on_frame` → `AppState::step_physics` → `AppState::render`, all bracketed by `Profiler` scopes.

`App` also tracks F7 as a chord modifier itself (not routed through `on_key`), so these always work regardless of the game's own key handler:
- **F7+P** — toggle the profiler HUD
- **F7+M** — toggle the mesh stats HUD

### Rendering pipeline

`AppState` (`runtime/src/core/app/app_state.rs`) owns all wgpu objects. It's a multi-pass pipeline now, not a single draw:

1. `on_frame` callback runs (user updates transforms), then physics steps (`PhysicsScene::step`)
2. `update_camera_pose()` pulls position/rotation off the first `(Transform, Camera)` entity
3. `accumulate_meshes()` (`rendering::mesh_helpers`) groups all `(Transform, Mesh)` entities by mesh ID into `InstanceRaw` batches (current + previous frame's model matrix, for TAA motion vectors), uploaded into a single pre-allocated `instance_buffer` (capacity 500,000 instances)
4. GPU-driven frustum culling: `cull.wgsl` (compute) reads `instance_buffer` and writes surviving instances into `visible_instance_buffer` plus per-mesh `draw_indexed_indirect` args (`CullingResources`, `runtime/src/core/rendering/culling.rs`). Hi-Z occlusion culling shaders (`hiz_copy.wgsl`, `hiz_downsample.wgsl`) exist but the hi-z pyramid rebuild is currently skipped since nothing reads it
5. Shadow pass: one depth-only render per cascade (4 cascades, CSM, `shadow_depth.wgsl` + `rendering::shadow`) into a `Depth32Float` texture array, sampled by the main pass with a comparison sampler
6. Main scene pass (`root.wgsl`): jittered (Halton(2,3) sequence, `taa_jitter`) draw of each mesh via `draw_indexed_indirect` from `visible_instance_buffer`, writing scene color (`Rgba16Float`) + per-pixel velocity (`Rg16Float`) offscreen targets
7. TAA resolve pass (`taa_resolve.wgsl`): reprojects the ping-ponged history buffer using the velocity buffer, neighborhood-clamps against the current frame, writes to both the swapchain and next frame's history slot
8. HUD text (glyphon) draws unjittered straight onto the swapchain in its own pass, so it stays crisp instead of being blurred by TAA — profiler HUD (F7+P) and mesh stats HUD (F7+M)

Shaders live under `runtime/src/core/app/shaders/`: `root.wgsl` (main), `shadow_depth.wgsl`, `taa_resolve.wgsl`, `cull.wgsl`, `hiz_copy.wgsl`, `hiz_downsample.wgsl`.

### ECS

Scenes are `bevy_ecs::world::World` instances stored in a `HashMap<String, World>` on `AppState`. The first scene created becomes the active rendering scene (`current_scene`).

`SceneHandle` (`runtime/src/core/ecs/scene_handle.rs`) is a thin wrapper around `&mut World` returned by `RuntimeContext::create_scene` / `get_scene`. Use `scene.query::<(...)>()`; use rayon's `par_iter_mut()` where the client needs parallel transform updates.

**ECS components** (all in `runtime/src/core/ecs/components/`):
- `Transform` — position (`Vec3`), rotation (`Quat`), scale (`Vec3`); converts to `Mat4` via `to_mat4()`
- `Mesh` — references a `MeshSourceId`; construct with `Mesh::cube()`, `Mesh::plane()`, or via `RuntimeContext::load_mesh(path)`
- `Camera` — marker component; the renderer uses the first entity with `(Transform, Camera)` as the camera
- `Material` — optional per-entity RGBA color (`Material::white/red/green/blue/black/new/from_vec4`), defaults to white if absent
- `RigidBody` — physics body (mass, restitution, `is_static`); requires `Transform` + `Collider` on the same entity. `handle` is filled in lazily by the physics step the first time it sees the entity
- `Collider` — collision shape: `Collider::Box { half_extents }` (or `Collider::cube()`) or `Collider::Inherit` (derives a convex hull from the entity's `Mesh`, works for built-in or asset-loaded meshes)

### Physics

Jolt Physics via `joltc-sys` (raw FFI) + `rolt` (safe wrapper) crates. `PhysicsScene` (`runtime/src/core/ecs/physics_system.rs`) owns the Jolt `PhysicsSystem`, a temp allocator, and a job system thread pool; one instance lives on `AppState` per scene, created once and stepped every frame.

- Only two object/broadphase layers exist: non-moving and moving (`OL_NON_MOVING`/`OL_MOVING`)
- `dt` is sourced from `AppState::tick_delta()` and clamped to `consts::MAX_PHYSICS_DT` so a stalled frame (window drag, resize) can't blow up integration
- Each physics step: registers any `RigidBody` without a cached Jolt handle yet, calls `system.update()`, then writes resulting positions/rotations back onto `Transform` (skips static bodies)
- Collider shapes are cached in `shape_cache: HashMap<(MeshSourceId, scale_bits, mass_bits), *mut JPC_Shape>` so entities sharing mesh+scale+mass (e.g. thousands of identical cubes) reuse one shape instead of rebuilding a convex hull each
- `Collider::Inherit` mass is baked into shape density via `aabb_volume` (bounding-box volume as a cheap stand-in for the hull's actual volume)
- Contains `unsafe` for the raw Jolt FFI calls (body creation, position/rotation readback, allocator/job-system lifetime) — each call site has a `// safety:` comment

### Mesh system

`MeshSourceId` is a `usize` newtype. Built-ins: `MeshSourceId::CUBE` (0) and `MeshSourceId::PLANE` (1), registered at startup by `MeshSourceHandler::init_default_meshes`. Custom meshes (loaded via `RuntimeContext::load_mesh` → `AssetReader::load_obj`, tobj-backed) get ids starting at 2 from `AppState::mesh_id_counter`, and are cached by file path (`mesh_path_cache`) so loading the same path twice reuses the existing GPU upload.

GPU mesh data (vertex + index buffers) lives in `AppState::mesh_registry`. `AppState::upload_mesh` also keeps CPU-side vertex positions (`mesh_points`) and a precomputed local-space bounding sphere (`mesh_bounds`) per mesh — `mesh_points` feeds `Collider::inherit()`'s convex hull build, `mesh_bounds` feeds `cull.wgsl`'s per-instance world-space bound.

### Assets

`runtime/src/core/assets/`: `AssetReader::load_obj` (tobj-backed .obj loader → `(Vec<VertexRaw>, Vec<u32>)`, single-indexed, missing normals/texcoords default to `(0,1,0)`/`(0,0)`) and `AssetReference` (path + `AssetReferenceType`, currently `Mesh` only).

### Profiling

`runtime/src/core/profiling/mod.rs`: `Profiler` is a general-purpose labeled start/stop timer registry with an insertion-ordered `entries()` iterator, exponential moving average smoothing, and arbitrary named extra values per label (instance counts, draw calls, etc). One instance lives on `AppState` and profiles engine internals (`frame`, `on_frame`, `physics`, `render`, `render/accumulate`, `render/scene_gpu`, `physics/register`, `physics/update`, `physics/sync`); `RuntimeContext::profile_scope(label)` exposes the same instance so game code can profile its own sections. Shown in the F7+P HUD via `ProfilerScope` RAII guards or explicit `start`/`stop`.

### Key constants (`runtime/src/consts.rs`)

- `MSAA_SAMPLE_COUNT = 1` — MSAA is currently disabled
- `DEFAULT_FOV = 45.0` degrees
- Camera far plane in `RenderCamera` is hardcoded to `10000.0` (overrides `DEFAULT_CAMERA_Z_FAR`, which goes unused)
- `MAX_PHYSICS_DT`, `MAX_PHYSICS_BODIES`, `PHYSICS_TEMP_ALLOCATOR_BYTES` — Jolt step clamp/capacity tuning
- `SHADOW_CASCADE_COUNT = 4`, `SHADOW_MAP_SIZE = 2048`, `SHADOW_MAX_DISTANCE = 600.0`, `SHADOW_SPLIT_LAMBDA = 0.6`, `SUN_DIRECTION_TO_LIGHT` — CSM shadow tuning (cascade count is also hardcoded in `root.wgsl`, keep both in sync if it changes)
- `MAX_CULL_MESH_SLOTS = 256` — max distinct meshes GPU-culled in one frame

`runtime/src/misc/config.rs` (`ViewportConfiguration`, `RuntimeConfiguration`) is a placeholder, not wired up to anything yet.
