use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

pub fn make_closure(original: &ItemFn) -> TokenStream {
    let body = &original.block;

    if original.sig.asyncness.is_some() {
        panic!("async feature is disabled")
    } else {
        quote! { (|| #body )() }
    }
}
