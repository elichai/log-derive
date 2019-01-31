#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
use proc_macro2::{Span, TokenStream, Punct};
use syn::{punctuated::Punctuated, token::Comma, buffer::TokenBuffer, *, token};
use quote::quote;
use syn::export::quote::ToTokens;
use log::Level;

// TODO: Change inner function to always be not visible
// TODO: Add loggings
// TODO: Let the user control the loggings.
// TODO: Add log_enabled condition to skip this all thing if not enabled.
fn fnargs_into_barename(list: &Punctuated<FnArg, Comma>) -> (TokenStream, bool) {
    let mut cool = Vec::with_capacity(list.len());
    let mut self_based = false;
    for arg in list {
        match arg {
            FnArg::Captured(ArgCaptured {pat, ..}) => {
                let a = pat.into_token_stream();
                let a: BareFnArgName = syn::parse2(a).expect("Failed on Bare1");
                cool.push(a);
            }
            FnArg::SelfRef(ArgSelfRef{self_token, ..}) | FnArg::SelfValue(ArgSelf{self_token, ..}) => {
                let ident = Ident::new("self", self_token.span);
                let bare = BareFnArgName::Named(ident);
                cool.push(bare);
                self_based = true;

            }
            _ => panic!("HAAAAAAAAA"),
        }
    }

    let args: Punctuated<BareFnArgName, Comma> = cool.into_iter().collect();
    (args.into_token_stream(), self_based)
}


fn set_logger(att: String) -> TokenStream {
    let mut att_str = att.chars();
    let att_str = att_str.next().unwrap().to_uppercase().to_string() + att_str.as_str();
    let mut res = Punctuated::new();
    res.push_value(Ident::new("log", Span::call_site()));
    res.push_punct(token::Colon2{ spans: [Span::call_site(); 2]});
    res.push_value(Ident::new("Level", Span::call_site()));
    res.push_punct(token::Colon2{ spans: [Span::call_site(); 2]});
    res.push_value(Ident::new(&att_str, Span::call_site()));
    res.into_token_stream()
}

fn get_ouside(inner_call: TokenStream, original: &ItemFn, logger: TokenStream) -> ItemFn {
    let outside_code = quote! {
        fn cool() {
                #inner_call
                log::log!(#logger, "LOG DERIVE: {:?}", res);
                res
        }
    };
    let mut outside: ItemFn = syn::parse2(outside_code).expect("Outside");
    outside.ident = Ident::new(&original.ident.to_string(), original.ident.span());
    outside.decl = original.decl.clone();
    outside.vis = original.vis.clone();

    outside
}

fn edit_original(mut original: ItemFn) -> ItemFn {
    original.ident = Ident::new(&format!("_{}", original.ident), original.ident.span());
    original.vis = Visibility::Inherited;
    original
}

#[proc_macro_attribute]
pub fn hello(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let attr2 = proc_macro2::TokenStream::from(attr.clone());
    let mut inside: ItemFn = parse_macro_input!(item as ItemFn);
    let (inputs, has_self) = fnargs_into_barename(&inside.decl.inputs);

    let logger = set_logger(attr2.to_string());
    let edited_inside = edit_original(inside.clone());
    let mut inner_call;
    if has_self {
        inner_call = quote!(let res = Self::#edited_inside.ident(#inputs););
    } else {
        inner_call = quote!(let res = #edited_inside.ident(#inputs););
    }
    let outside = get_ouside(inner_call, &inside, logger);

    println!("{:#?}", inside);
    let res = quote! {
        #edited_inside
        #outside
    };
//    println!("{}", res);
    res.into()

}
