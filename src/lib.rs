#![recursion_limit="128"]
extern crate proc_macro;
extern crate syn;
use proc_macro2::{Span, TokenStream};
use syn::{punctuated::Punctuated, *, token};
use quote::quote;
use syn::export::quote::ToTokens;

// TODO: Let the user Customize loggings.
// TODO: Add log_enabled condition to skip this all thing if not enabled.
// TODO: Optimize imports. and optimize syn features.
// TODO: How should I do this with traits impl?
// TODO: Add different log levels for Ok/Err (if the return value is Result)

fn set_logger(att: &TokenStream) -> TokenStream {
    let attr = att.to_string().to_lowercase();
    let mut attr_char = attr.chars();
    let att_str = attr_char.next().unwrap().to_uppercase().to_string() + attr_char.as_str();
    let mut res = Punctuated::new();
    res.push_value(Ident::new("log", Span::call_site()));
    res.push_punct(token::Colon2{ spans: [Span::call_site(); 2]});
    res.push_value(Ident::new("Level", Span::call_site()));
    res.push_punct(token::Colon2{ spans: [Span::call_site(); 2]});
    res.push_value(Ident::new(&att_str, Span::call_site()));
    res.into_token_stream()
}

fn make_closure(original: &ItemFn) -> ExprClosure {
    let body = Box::new(Expr::Block(ExprBlock{
        attrs: Default::default(),
        label: Default::default(),
        block: *original.block.clone(),
    }));

    ExprClosure{
        attrs: Default::default(),
        asyncness: Default::default(),
        movability: Default::default(),
        capture: Default::default(),
        or1_token: Default::default(),
        inputs: Default::default(),
        or2_token: Default::default(),
        output: ReturnType::Default,
        body,
    }
}

fn replace_function_headers(original: &ItemFn, new: &mut ItemFn) {
    new.ident = Ident::new(&original.ident.to_string(), original.ident.span());
    new.decl = original.decl.clone();
    new.vis = original.vis.clone();
}

fn generate_function(closure: &ExprClosure, logger: &TokenStream) -> Result<ItemFn> {
    let code = quote!{
        fn temp() {
            let mut closure = #closure;
            let result = closure();
            log::log!(#logger, "LOG DERIVE: {:?}", result);
            result
        }
    };
     syn::parse2(code)
}

#[proc_macro_attribute]
pub fn logfn(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let original_fn: ItemFn = parse_macro_input!(item as ItemFn);

    let closure = make_closure(&original_fn);
    let logger = set_logger(&attr);
    let mut new_fn = generate_function(&closure, &logger).expect("Failed Generating Function");
    replace_function_headers(&original_fn, &mut new_fn);

    new_fn.into_token_stream().into()

}
