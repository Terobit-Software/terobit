//! This library contains proc-macros for the v1 SDK.

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

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
#[proc_macro_attribute]
pub fn main(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;

    quote! {
        #input_fn

        #[unsafe(no_mangle)]
        pub extern "C" fn _terobit_v1_main() {
            #fn_name();
        }
    }
    .into()
}

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
#[proc_macro_attribute]
pub fn pre_physics(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;

    quote! {
        #input_fn

        #[unsafe(no_mangle)]
        pub extern "C" fn _terobit_v1_pre_physics_update() {
            #fn_name();
        }
    }
    .into()
}

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
#[proc_macro_attribute]
pub fn post_physics(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;

    quote! {
        #input_fn

        #[unsafe(no_mangle)]
        pub extern "C" fn _terobit_v1_post_physics_update() {
            #fn_name();
        }
    }
    .into()
}

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
#[proc_macro_attribute]
pub fn pre_render(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;

    quote! {
        #input_fn

        #[unsafe(no_mangle)]
        pub extern "C" fn _terobit_v1_pre_render_update() {
            #fn_name();
        }
    }
    .into()
}

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
#[proc_macro_attribute]
pub fn post_render(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;

    quote! {
        #input_fn

        #[unsafe(no_mangle)]
        pub extern "C" fn _terobit_v1_post_render_update() {
            #fn_name();
        }
    }
    .into()
}
