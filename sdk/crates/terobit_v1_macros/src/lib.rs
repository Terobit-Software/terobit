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
pub fn update(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;

    quote! {
        #input_fn

        #[unsafe(no_mangle)]
        pub extern "C" fn _terobit_v1_update() {
            #fn_name();
        }
    }
    .into()
}
