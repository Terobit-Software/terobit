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
    /// `fn()` -> `()`
    ///
    /// (nothing) -> (nothing)
    ///
    /// ```rust
    /// use terobit_v1::main;
    ///
    /// #[main]
    /// fn main () {
    ///     println!("This code is run when the script is loaded.");
    /// }
    /// ```
    main(MAIN_ENTRYPOINT)
);

entrypoint!(
    /// Called before rendering and physics. Update stage 1 (first).
    ///
    /// `fn()` -> `()`
    ///
    /// (nothing) -> (nothing)
    ///
    /// ```rust
    /// use terobit_v1::update;
    ///
    /// #[update::pre_physics]
    /// fn pre_physics () {
    ///     println!("This code is run every frame BEFORE physics.");
    /// }
    /// ```
    pre_physics(PRE_PHYSICS_ENTRYPOINT)
);

entrypoint!(
    /// Called every frame after physics, but before pre-rendering. Update stage 2.
    ///
    /// `fn()` -> `()`
    ///
    /// (nothing) -> (nothing)
    ///
    /// ```rust
    /// use terobit_v1::update;
    ///
    /// #[update::post_physics]
    /// fn post_physics () {
    ///     println!("This code is run every frame AFTER physics.");
    /// }
    /// ```
    post_physics(POST_PHYSICS_ENTRYPOINT)
);

entrypoint!(
    /// Called every frame before rendering, but after post-physics. Update stage 3.
    ///
    /// `fn()` -> `()`
    ///
    /// (nothing) -> (nothing)
    ///
    /// ```rust
    /// use terobit_v1::update;
    ///
    /// #[update::pre_render]
    /// fn pre_render () {
    ///     println!("This code is run every frame BEFORE rendering.");
    /// }
    /// ```
    pre_render(PRE_RENDER_ENTRYPOINT)
);

entrypoint!(
    /// Called every frame after rendering and physics. Update stage 4 (final).
    ///
    /// `fn()` -> `()`
    ///
    /// (nothing) -> (nothing)
    ///
    /// ```rust
    /// use terobit_v1::update;
    ///
    /// #[update::post_render]
    /// fn post_render () {
    ///     println!("This code is run every frame AFTER rendering.");
    /// }
    /// ```
    post_render(POST_RENDER_ENTRYPOINT)
);
