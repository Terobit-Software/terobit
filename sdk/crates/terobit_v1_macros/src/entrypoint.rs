//! Function for expansion of entry functions.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, parse::Nothing, parse_macro_input};

pub(crate) fn expand(export_name: &str, args: TokenStream, item: TokenStream) -> TokenStream {
    parse_macro_input!(args as Nothing);
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let export_ident = format_ident!("{}", export_name);

    quote! {
        #input_fn

        #[unsafe(no_mangle)]
        pub extern "C" fn #export_ident() {
            #fn_name();
        }
    }
    .into()
}
