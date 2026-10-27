use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

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
