/// Called after the script is loaded and host resources are initialized.
///
/// `fn()` -> `()`
///
/// (nothing) -> (nothing)
///
/// ```rust
/// use terobit_v1::prelude::*;
///
/// #[main]
/// fn main () {
///     log("This code is run when the script is loaded.");
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
/// `fn()` -> `()`
///
/// (nothing) -> (nothing)
///
/// ```rust
/// use terobit_v1::prelude::*;
///
/// #[update::pre_render]
/// fn pre_render () {
///     log("This code is run every frame BEFORE rendering.");
/// }
/// ```
pub const PRE_RENDER_ENTRYPOINT: &str = "_terobit_v1_pre_render_update";

/// Called every frame after rendering and physics. Update stage 4 (final).
///
/// `fn()` -> `()`
///
/// (nothing) -> (nothing)
///
/// ```rust
/// use terobit_v1::prelude::*;
///
/// #[update::post_rendering]
/// fn post_rendering () {
///     log("This code is run every frame AFTER rendering.");
/// }
/// ```
pub const POST_RENDER_ENTRYPOINT: &str = "_terobit_v1_post_render_update";

/// Called before rendering and physics. Update stage 1 (first).
///
/// `fn()` -> `()`
///
/// (nothing) -> (nothing)
///
/// ```rust
/// use terobit_v1::prelude::*;
///
/// #[update::pre_physics]
/// fn pre_physics () {
///     log("This code is run every frame BEFORE physics.");
/// }
/// ```
pub const PRE_PHYSICS_ENTRYPOINT: &str = "_terobit_v1_pre_physics_update";

/// Called every frame after physics, but before pre-rendering. Update stage 2.
///
/// `fn()` -> `()`
///
/// (nothing) -> (nothing)
///
/// ```rust
/// use terobit_v1::prelude::*;
///
/// #[update::post_physics]
/// fn post_physics () {
///     log("This code is run every frame AFTER physics.");
/// }
/// ```
pub const POST_PHYSICS_ENTRYPOINT: &str = "_terobit_v1_post_physics_update";

// command buffer:
//   at the end of every cycle the host reads `len` bytes from `ptr`,
//   applies them, then calls `clear`.

/// Pointer to the start of the command buffer in guest memory.
///
/// `fn()` -> `i32`
///
/// (nothing) -> (pointer to command buffer)
pub const COMMANDS_PTR_ENDPOINT: &str = "_terobit_v1_commands_ptr";

/// Byte length of the command buffer.
///
/// `fn()` -> `i32`
///
/// (nothing) -> (byte length of command buffer)
pub const COMMANDS_LEN_ENDPOINT: &str = "_terobit_v1_commands_len";

/// Empties the command buffer. Called by the host after it has read it.
///
/// `fn()` -> `()`
///
/// (nothing) -> (nothing)
pub const COMMANDS_CLEAR_ENDPOINT: &str = "_terobit_v1_commands_clear";
