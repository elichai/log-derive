#![recursion_limit="128"]
extern crate proc_macro;
extern crate syn;
use proc_macro2::{Span, TokenStream};
use syn::{punctuated::Punctuated, *, token};
use quote::quote;
use syn::export::quote::ToTokens;

// TODO: Optimize imports. and optimize syn features.

struct FormattedAttributes {
    ok_expr: TokenStream,
    err_expr: TokenStream,
}


impl FormattedAttributes {
    pub fn parse_attributes(attr: AttributeArgs) -> Self {
        #[derive(Default, Debug)]
        struct Attributes {
            ok_log: Option<String>,
            err_log: Option<String>,
            fmt: Option<String>,
            general_log: Option<String>
        }
        let mut result = Attributes::default();
        for at in attr {
            match at {
                NestedMeta::Meta(meta) => {
                    match meta {
                        Meta::Word(ident) => result.general_log = Some(ident.to_string()),
                        Meta::NameValue(nv) => {
                            match nv.ident.to_string().to_lowercase().as_str() {
                                "fmt" => result.fmt = Some(get_literal_str(nv.lit)),
                                "ok" => result.ok_log = Some(get_literal_str(nv.lit)),
                                "err" => result.err_log = Some(get_literal_str(nv.lit)),
                                _ => panic!("Unsupported literal"),
                            }
                        }
                        Meta::List(_) => panic!("List in the macro aren't supported"),
                    }
                },
                NestedMeta::Literal(_) => panic!("Direct literals  outside of name=value aren't supported"),
            }
        }
        return get_ok_err_streams(result);


        fn get_ok_err_streams(att: Attributes) -> FormattedAttributes {
            let Attributes { ok_log, err_log, fmt, general_log } = att;

            let ok_log = ok_log.map_or_else(|| general_log.clone(), Some );
            let err_log = err_log.map_or_else(|| general_log, Some);
            let fmt = fmt.unwrap_or_else(|| String::from("LOG DERIVE: {:?}"));

            let ok_expr = match ok_log {
                Some(loglevel) => {
                    let log_token = get_logger_token(&loglevel);
                    quote!{log::log!(#log_token, #fmt, result);}
                }
                None => quote!{()},
            };

            let err_expr = match err_log {
                Some(loglevel) => {
                    let log_token = get_logger_token(&loglevel);
                    quote!{log::log!(#log_token, #fmt, err);}
                }
                None => quote!{()},
            };
            FormattedAttributes { ok_expr, err_expr }
        }
    }
}

fn get_literal_str(l: Lit) -> String {
    match l {
        Lit::Str(litstr) => litstr.value(),
        _ => panic!("Literals other the Str aren't supported"),
    }
}

fn check_if_return_result(f: &ItemFn) -> bool {
    let retrn = &f.decl.output;
    match retrn {
        ReturnType::Default => false,
        ReturnType::Type(_, t) => {
            match  *t.clone() {
                Type::Path(tp) => {
                    if tp.path.segments.is_empty() {
                        false
                    } else {
                        tp.path.segments[0].ident == "Result"
                    }
                },
                _ => false,
            }
        }

    }

}


fn get_logger_token(att: &str) -> TokenStream {
    let attr = att.to_lowercase();
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

fn generate_function(closure: &ExprClosure, expressions: &FormattedAttributes, result: bool) -> Result<ItemFn> {
    let FormattedAttributes { ok_expr, err_expr } = expressions;
    let code = if result {
        quote!{
            fn temp() {
                let mut closure = #closure;
                match closure() {
                    Ok(result) => {
                        #ok_expr;
                        return Ok(result);
                    }
                    Err(err) => {
                        #err_expr;
                        return Err(err);
                    }
                }
            }
        }
    } else {
        quote!{
            fn temp() {
                let mut closure = #closure;
                let result = closure();
                #ok_expr;
                result
            }
        }
    };

     syn::parse2(code)
}
#[proc_macro_attribute]
pub fn logfn(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let attr = parse_macro_input!(attr as AttributeArgs);
    let parsed_attributes = FormattedAttributes::parse_attributes(attr);
    let original_fn: ItemFn = parse_macro_input!(item as ItemFn);

    let closure = make_closure(&original_fn);
    let is_result = check_if_return_result(&original_fn);
    let mut new_fn = generate_function(&closure, &parsed_attributes, is_result).expect("Failed Generating Function");
    replace_function_headers(&original_fn, &mut new_fn);
    new_fn.into_token_stream().into()

}
