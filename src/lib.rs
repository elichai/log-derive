#![recursion_limit="128"]

//! # Log Derive
//!
//! `log-derive` provides a simple attribute macro that facilitates logs as part of the [`log`] facade <br>
//! Right now the only macro is [`logfn`], this macro is only for functions but it still have a lot of power.
//!
//!
//!  # Use
//! The basic use of the macro is by putting it on top of the function like this: `#[logfn(INFO)]` <br>
//! The return type of the function **must** implement Debug in order for this to work. <br>
//! The macro will accept all log levels provided by the [`log`] facade. <br>
//! If the function return a [`Result`] type the macro will accept the following additional attributes:
//! `(ok = "LEVEL")` and `(err = "LEVEL")` this can provide different log levels if the function failed or not. <br>
//! By default the macro uses the following formatting to print the message: `("LOG DERIVE: {:?}", return_val)` <br>
//! This can be easily changed using the `fmt` attribute: `#[logfn(LEVEL, fmt = "Important Result: {:}")`
//! which will accept format strings similar to [`println!`].
//!
//! [`logfn`]: ./attr.logfn.html
//! [`log`]: https://docs.rs/log/latest/log/index.html
//! [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
//! [`println!`]: https://doc.rust-lang.org/stable/std/macro.println.html
//!
//! ## Examples
//! ```rust
//! #[macro_use]
//! extern crate log_derive;
//! #[macro_use]
//! extern crate log;
//!
//! # #[derive(Debug)]
//! struct Error;
//! # #[derive(Debug)]
//! struct Success;
//! # #[derive(Debug)]
//! enum Status { Alive, Dead, Unknown }
//!
//! #[logfn(Warn)]
//! fn is_alive(person: &Person) -> Status {
//!     # use self::Response::*;
//!     # use self::Status::*;
//!    match person.ping() {
//!        Pong => Status::Alive,
//!        Timeout => if person.is_awake() {
//!            Unknown
//!        } else {
//!            Dead
//!        }
//!   }
//!}
//!
//! #[logfn(ok = "TRACE", err = "ERROR")]
//! fn call_isan(num: &str) -> Result<Success, Error> {
//!     if num.len() >= 10 && num.len() <= 15 {
//!         Ok(Success)
//!     } else {
//!         Err(Error)
//!     }
//! }
//!
//! #[logfn(INFO, fmt = "a + b = {}")]
//! fn addition(a: usize, b: usize) -> usize {
//!     a + b
//! }
//!
//! # fn main() {}
//! # enum Response {Pong, Timeout}
//! # struct Person;
//! # impl Person {fn ping(&self) -> Response {Response::Pong}fn is_awake(&self) -> bool {true}}
//! ```
//!
//!
extern crate proc_macro;
extern crate syn;
use proc_macro2::{Span, TokenStream};
use syn::{parse_macro_input, AttributeArgs, NestedMeta, Meta, ReturnType, Ident, ItemFn, Result,
          Expr, ExprClosure, ExprBlock, Lit, token, Type, punctuated::Punctuated, export::quote::ToTokens, spanned::Spanned};
use quote::quote;

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

/// Check if a return type is some form of `Result`. This assumes that all types named `Result`
/// are in fact results, but is resilient to the possibility of `Result` types being referenced
/// from specific modules.
pub(crate) fn is_result_type(ty: &syn::TypePath) -> bool {
    if let Some(segment) = ty.path.segments.iter().last() {
        segment.ident == "Result"
    } else {
        false
    }
}

fn check_if_return_result(f: &ItemFn) -> bool {
    if let ReturnType::Type(_, t) = &f.decl.output {
        if let Type::Path(path) = t.as_ref() {
            return is_result_type(path);
        }
    }

    false
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
        capture: Some(token::Move{span: original.span()}),
        or1_token: Default::default(),
        inputs: Default::default(),
        or2_token: Default::default(),
        output: ReturnType::Default,
        body,
    }
}

fn replace_function_headers(original: ItemFn, new: &mut ItemFn) {
    let block = new.block.clone();
    *new = original;
    new.block = block;
}

fn generate_function(closure: &ExprClosure, expressions: &FormattedAttributes, result: bool) -> Result<ItemFn> {
    let FormattedAttributes { ok_expr, err_expr } = expressions;
    let code = if result {
        quote!{
            fn temp() {
                let result = (#closure)();
                match result {
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
                let result = (#closure)();
                #ok_expr;
                result
            }
        }
    };

     syn::parse2(code)
}

/// Logs the result of the function it's above.
/// # Examples
/// ``` rust
///  # #[macro_use] extern crate log_derive;
/// # use std::{net::*, io::{self, Write}};
/// #[logfn(Err = "Error", fmt = "Failed Sending Packet: {:?}")]
/// fn send_hi(addr: SocketAddr) -> Result<(), io::Error> {
///     let mut stream = TcpStream::connect(addr)?;
///     stream.write(b"Hi!")?;
///     Ok( () )
/// }
///
///
/// ```
#[proc_macro_attribute]
pub fn logfn(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let attr = parse_macro_input!(attr as AttributeArgs);
    let parsed_attributes = FormattedAttributes::parse_attributes(attr);
    let original_fn: ItemFn = parse_macro_input!(item as ItemFn);

    let closure = make_closure(&original_fn);
    let is_result = check_if_return_result(&original_fn);
    let mut new_fn = generate_function(&closure, &parsed_attributes, is_result).expect("Failed Generating Function");
    replace_function_headers(original_fn, &mut new_fn);
    new_fn.into_token_stream().into()

}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::parse_quote;

    use super::is_result_type;

    #[test]
    fn result_type() {
        assert!(is_result_type(&parse_quote!(Result<T, E>)));
        assert!(is_result_type(&parse_quote!(std::result::Result<T, E>)));
        assert!(is_result_type(&parse_quote!(fmt::Result)));
    }
}