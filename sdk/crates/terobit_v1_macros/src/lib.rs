//! This library contains proc-macros for the v1 SDK.
//!
//! Every macro here is an entrypoint attribute, declared once with
//! `entrypoint!` against its export name in `terobit_v1_abi::endpoints`.

mod entrypoint;

use proc_macro::TokenStream;
use terobit_v1_abi::endpoints::*;

/// Declares an entrypoint attribute macro that exports the annotated fn
/// under the given `terobit_v1_abi::endpoints` constant.
///
/// ```ignore
/// entrypoint!(
///     /// docs
///     main(MAIN_ENTRYPOINT)
/// );
/// ```
macro_rules! entrypoint {
    ($(#[$attr:meta])* $name:ident($export:ident)) => {
        $(#[$attr])*
        #[proc_macro_attribute]
        pub fn $name(args: TokenStream, item: TokenStream) -> TokenStream {
            entrypoint::expand($export, args, item)
        }
    };
}

entrypoint!(
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
    main(MAIN_ENTRYPOINT)
);

entrypoint!(
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
    pre_physics(PRE_PHYSICS_ENTRYPOINT)
);

entrypoint!(
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
    post_physics(POST_PHYSICS_ENTRYPOINT)
);

entrypoint!(
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
    pre_render(PRE_RENDER_ENTRYPOINT)
);

entrypoint!(
    /// Called every frame after rendering and physics. Update stage 4 (final).
    ///
    /// `fn(FrameInfo)` -> `u16`
    ///
    /// (info for this frame) -> post render fn exit code
    ///
    /// ```rust
    /// use terobit_v1::update;
    ///
    /// #[update::post_render]
    /// fn post_render (info: FrameInfo) -> u16 {
    ///     println!("This code is run every frame AFTER rendering.");
    ///     0
    /// }
    /// ```
    post_render(POST_RENDER_ENTRYPOINT)
);
