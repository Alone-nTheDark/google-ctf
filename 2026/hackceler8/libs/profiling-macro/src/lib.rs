extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemFn);
    let fn_name = input.sig.ident.to_string();

    let struct_name = attr.to_string().trim().to_string();
    let hash_input = if struct_name.is_empty() {
        fn_name
    } else {
        format!("{}::{}", struct_name, fn_name)
    };

    // Stable FNV-1a Hash (32-bit version)
    let mut hash: u32 = 2166136261;
    for byte in hash_input.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(16777619);
    }

    let id = (hash & 0x7FFF) as u16; // 15 bits for ID, 1 bit for flag

    let block = &input.block;

    // Wrap the original function body
    input.block = syn::parse2(quote! {
        {
            // Calls the start reporting function + reports end of function when dropped.
            #[cfg(feature = "profiling")]
            let _guard = ::profiling::ProfilingGuard::for_function_id(#id);
            #block
        }
    })
    .unwrap();

    TokenStream::from(quote! { #input })
}
