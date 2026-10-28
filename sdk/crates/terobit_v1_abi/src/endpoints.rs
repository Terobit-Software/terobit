/// Called after the script is loaded and host resources are initialized.
///
/// `fn()` -> `u16`
///
/// (nothing) -> main fn exit code
///
/// ```rust
/// use terobit_v1::main;
///
/// #[main]
/// fn main () -> u16 {
///     println!("This code is run when the script is loaded.");
///     0
/// }
/// ```
pub const MAIN_ENTRYPOINT: &str = "_terobit_v1_main";

// update order:
//   1. pre-physics
//   2. post-physics
//   3. pre-render
//   4. post-render

/// Called every frame before rendering, but after post-physics. Update stage 3.
///
/// `fn(FrameInfo)` -> `u16`
///
/// (info for this frame) -> pre render fn exit code
///
/// ```rust
/// use terobit_v1::update;
///
/// #[update::pre_render]
/// fn pre_render (info: FrameInfo) -> u16 {
///     println!("This code is run every frame BEFORE rendering.");
///     0
/// }
/// ```
pub const PRE_RENDER_ENTRYPOINT: &str = "_terobit_v1_pre_render_update";

/// Called every frame after rendering and physics. Update stage 4 (final).
///
/// `fn(FrameInfo)` -> `u16`
///
/// (info for this frame) -> post render fn exit code
///
/// ```rust
/// use terobit_v1::update;
///
/// #[update::post_rendering]
/// fn post_rendering (info: FrameInfo) -> u16 {
///     println!("This code is run every frame AFTER rendering.");
///     0
/// }
/// ```
pub const POST_RENDER_ENTRYPOINT: &str = "_terobit_v1_post_render_update";

/// Called before rendering and physics. Update stage 1 (first).
///
/// `fn(FrameInfo)` -> `u16`
///
/// (info for this frame) -> pre physics fn exit code
///
/// ```rust
/// use terobit_v1::update;
///
/// #[update::pre_physics]
/// fn pre_physics (info: FrameInfo) -> u16 {
///     println!("This code is run every frame BEFORE physics.");
///     0
/// }
/// ```
pub const PRE_PHYSICS_ENTRYPOINT: &str = "_terobit_v1_pre_physics_update";

/// Called every frame after physics, but before pre-rendering. Update stage 2.
///
/// `fn(FrameInfo)` -> `u16`
///
/// (info for this frame) -> post physics fn exit code
///
/// ```rust
/// use terobit_v1::update;
///
/// #[update::post_physics]
/// fn post_physics (info: FrameInfo) -> u16 {
///     println!("This code is run every frame AFTER physics.");
///     0
/// }
/// ```
pub const POST_PHYSICS_ENTRYPOINT: &str = "_terobit_v1_post_physics_update";
