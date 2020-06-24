#![recursion_limit = "128"]

//! # Log Derive
//!
//! `log-derive` provides a simple attribute macro that facilitates logs as part of the [`log`] facade <br>
//! Right now it contains two macros [`logfn`], [`logfn_inputs`] these macros are only for functions but still have a lot of power.
//!
//!
//!  # Use
//! The basic use of these macros is by putting one or both of them on top of the function like this: `#[logfn(INFO)]` <br>
//!
//! The [`logfn`] macro is used to log the *output* of the function and [`logfn_inputs`] is used to log the *inputs*. <br>
//! Please notice, the arguments being logged **must** implement the [`Debug`] trait. <br>
//! (i.e. [`logfn`] requires the output to be [`Debug`] and [`logfn_inputs`] require the inputs to be [`Debug`]) <br>
//!
//! The macros will accept all log levels provided by the [`log`] facade. <br>
//! In [`logfn`] if the function returns a [`Result`] type the macro will accept the following additional attributes: <br>
//! `(ok = "LEVEL")` and `(err = "LEVEL")` this can provide different log levels if the function failed or not. <br>
//!
//! By default the macro uses the following formatting to print the message: <br>
//! [`logfn`]: `("FUNCTION_NAME() => {:?}", return_val)` <br>
//! [`logfn_inputs`]: `"FUNCTION_NAME(a: {:?}, b: {:?})", a, b)` <br>
//! This can be easily changed using the `fmt` attribute: `#[logfn(LEVEL, fmt = "Important Result: {:}")` <br>
//! which will accept format strings similar to [`println!`].
//!
//! [`logfn`]: ./attr.logfn.html
//! [`logfn_inputs`]: ./attr.logfn_inputs.html
//! [`log`]: https://docs.rs/log/latest/log/index.html
//! [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
//! [`println!`]: https://doc.rust-lang.org/stable/std/macro.println.html
//! [`Debug`]: https://doc.rust-lang.org/std/fmt/trait.Debug.html
//!
//! ## Examples
//! ```rust
//! use log_derive::{logfn, logfn_inputs};
//!
//! # #[derive(Debug)]
//! struct Error;
//! # #[derive(Debug)]
//! struct Success;
//! # #[derive(Debug)]
//! enum Status { Alive, Dead, Unknown }
//!
//! #[logfn(Warn)]
//! #[logfn_inputs(Info, fmt = "Checking if {:?} is alive")]
//! fn is_alive(person: &Person) -> Status {
//!     # use Response::*;
//!     # use Status::*;
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
//! #[logfn_inputs(Info)]
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
//! #[logfn_inputs(Trace, fmt = "adding a: {:?} and b: {:?}")]
//! fn addition(a: usize, b: usize) -> usize {
//!     a + b
//! }
//!
//! #[logfn_inputs(Info)]
//! #[logfn(ok = "TRACE", log_ts = true)]
//! fn time_this(num: &str) -> Result<Success, Error> {
//!     if num.len() >= 10 && num.len() <= 15 {
//!        std::thread::sleep(Duration::from_secs(1));
//!         Ok(Success)
//!     } else {
//!         Err(Error)
//!     }
//! }
//!
//! # enum Response {Pong, Timeout}
//! # #[derive(Debug)]
//! # struct Person;
//! # impl Person {fn ping(&self) -> Response {Response::Pong}fn is_awake(&self) -> bool {true}}
//! # use std::time::Duration;
//! ```
//!
//!
extern crate proc_macro;
extern crate syn;
use darling::{Error, FromMeta};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use syn::punctuated::Punctuated;
use syn::{
    parse_macro_input, token, AttributeArgs, FnArg, Ident, ItemFn, Meta, NestedMeta, Pat, Result, ReturnType, Stmt, Type, TypePath,
};

#[cfg(feature = "async")]
mod async_closure;

#[cfg(feature = "async")]
use async_closure::make_closure;

#[cfg(not(feature = "async"))]
mod non_async_closure;

#[cfg(not(feature = "async"))]
use non_async_closure::make_closure;

struct FormattedAttributes {
    ok_expr: TokenStream,
    err_expr: TokenStream,
    log_ts: bool,
    contained_ok_or_err: bool,
}

impl FormattedAttributes {
    pub fn parse_attributes(attr: &[NestedMeta], fmt_default: String) -> darling::Result<Self> {
        OutputOptions::from_list(attr).map(|opts| Self::get_ok_err_streams(opts, fmt_default))
    }

    fn get_ok_err_streams(att: OutputOptions, fmt_default: String) -> Self {
        let contained_ok_or_err = att.contains_ok_or_err();
        let log_ts = att.log_ts();
        let ok_log = att.ok_log();
        let err_log = att.err_log();
        let mut fmt = att.fmt().unwrap_or(fmt_default);
        if log_ts {
            fmt += ", ts={:#?}"
        };

        let ok_expr = match ok_log {
            Some(loglevel) => {
                let log_token = get_logger_token(&loglevel);
                if log_ts {
                    quote! {log::log!(#log_token, #fmt, result, ts); }
                } else {
                    quote! {log::log!(#log_token, #fmt, result); }
                }
            }
            None => quote! {()},
        };

        let err_expr = match err_log {
            Some(loglevel) => {
                let log_token = get_logger_token(&loglevel);
                if log_ts {
                    quote! {log::log!(#log_token, #fmt, err, ts); }
                } else {
                    quote! {log::log!(#log_token, #fmt, err); }
                }
            }
            None => quote! {()},
        };
        FormattedAttributes { ok_expr, err_expr, log_ts, contained_ok_or_err }
    }
}

#[derive(Default, FromMeta)]
#[darling(default)]
struct OutputNamedOptions {
    ok: Option<Ident>,
    err: Option<Ident>,
    fmt: Option<String>,
    log_ts: Option<bool>,
}

struct OutputOptions {
    /// The log level specified as the first word in the attribute.
    leading_level: Option<Ident>,
    named: OutputNamedOptions,
}

struct InputOptions {
    level: Ident,
    fmt: Option<String>,
}

impl FromMeta for InputOptions {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        let level;
        let mut fmt = None;
        if items.is_empty() {
            return Err(Error::too_few_items(1));
        }

        match &items[0] {
            NestedMeta::Meta(first) => {
                if let Meta::Path(path) = first {
                    if let Some(ident) = path.get_ident() {
                        level = ident.clone();
                    } else {
                        return Err(Error::unexpected_type("first item should be a log level"));
                    }
                } else {
                    return Err(Error::unexpected_type("first item should be a log level"));
                }
            }
            NestedMeta::Lit(lit) => return Err(Error::unexpected_lit_type(lit)),
        }

        if items.len() > 1 {
            fmt = String::from_nested_meta(&items[1]).ok();
        }

        Ok(InputOptions { level, fmt })
    }
}

impl OutputOptions {
    pub fn ok_log(&self) -> Option<&Ident> {
        self.named.ok.as_ref().or_else(|| self.leading_level.as_ref())
    }

    pub fn err_log(&self) -> Option<&Ident> {
        self.named.err.as_ref().or_else(|| self.leading_level.as_ref())
    }

    pub fn contains_ok_or_err(&self) -> bool {
        self.named.ok.is_some() || self.named.err.is_some()
    }

    pub fn log_ts(&self) -> bool {
        self.named.log_ts.unwrap_or(false)
    }

    pub fn fmt(&self) -> Option<String> {
        self.named.fmt.clone()
    }
}

impl FromMeta for OutputOptions {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        if items.is_empty() {
            return Err(darling::Error::too_few_items(1));
        }

        let mut leading_level = None;

        if let NestedMeta::Meta(first) = &items[0] {
            if let Meta::Path(path) = first {
                leading_level = path.get_ident().cloned();
            }
        }

        let named =
            if leading_level.is_some() { OutputNamedOptions::from_list(&items[1..])? } else { OutputNamedOptions::from_list(items)? };

        Ok(OutputOptions { leading_level, named })
    }
}

/// Check if a return type is some form of `Result`. This assumes that all types named `Result`
/// are in fact results, but is resilient to the possibility of `Result` types being referenced
/// from specific modules.
pub(crate) fn is_result_type(ty: &TypePath) -> bool {
    if let Some(segment) = ty.path.segments.iter().last() {
        segment.ident == "Result"
    } else {
        false
    }
}

fn check_if_return_result(f: &ItemFn) -> bool {
    if let ReturnType::Type(_, t) = &f.sig.output {
        return match t.as_ref() {
            Type::Path(path) => is_result_type(path),
            _ => false,
        };
    }

    false
}

fn get_logger_token(att: &Ident) -> TokenStream {
    // Capitalize the first letter.
    let attr_str = att.to_string().to_lowercase();
    let mut attr_char = attr_str.chars();
    let attr_str = attr_char.next().unwrap().to_uppercase().to_string() + attr_char.as_str();
    let att_str = Ident::new(&attr_str, att.span());
    quote!(log::Level::#att_str)
}

fn replace_function_headers(original: ItemFn, new: &mut ItemFn) {
    let block = new.block.clone();
    *new = original;
    new.block = block;
}

fn generate_function(closure: &TokenStream, expressions: FormattedAttributes, result: bool) -> Result<ItemFn> {
    let FormattedAttributes { ok_expr, err_expr, log_ts, contained_ok_or_err } = expressions;
    let result = result || contained_ok_or_err;
    let code = if log_ts {
        if result {
            quote! {
                fn temp() {
                    let instant = std::time::Instant::now();
                    let result = #closure;
                    let ts = instant.elapsed();
                    result.map(|result| { #ok_expr; result })
                        .map_err(|err| { #err_expr; err })
                }
            }
        } else {
            quote! {
                fn temp() {
                    let instant = std::time::Instant::now();
                    let result = #closure;
                    let ts = instant.elapsed();
                    #ok_expr;
                    result
                }
            }
        }
    } else if result {
        quote! {
            fn temp() {
                let result = #closure;
                result.map(|result| { #ok_expr; result })
                    .map_err(|err| { #err_expr; err })
            }
        }
    } else {
        quote! {
            fn temp() {
                let result = #closure;
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
/// #[logfn(err = "Error", fmt = "Failed Sending Packet: {:?}")]
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
    let original_fn: ItemFn = parse_macro_input!(item as ItemFn);
    let fmt_default = original_fn.sig.ident.to_string() + "() => {:?}";
    let parsed_attributes: FormattedAttributes = match FormattedAttributes::parse_attributes(&attr, fmt_default) {
        Ok(val) => val,
        Err(err) => {
            return err.write_errors().into();
        }
    };
    let closure = make_closure(&original_fn);
    let is_result = check_if_return_result(&original_fn);
    let mut new_fn = generate_function(&closure, parsed_attributes, is_result).expect("Failed Generating Function");
    replace_function_headers(original_fn, &mut new_fn);
    new_fn.into_token_stream().into()
}

/// Logs the inputs of the function
/// # Examples
/// ``` rust
///  # #[macro_use] extern crate log_derive;
/// # use std::{net::*, io::{self, Write}};
/// #[logfn_inputs(INFO, fmt = "Good morning: {:?}, to: {:?}")]
/// fn good_morning(msg: &str, addr: SocketAddr) -> Result<(), io::Error> {
///     let mut stream = TcpStream::connect(addr)?;
///     stream.write(msg.as_bytes())?;
///     Ok( () )
/// }
///
///
/// ```
#[proc_macro_attribute]
pub fn logfn_inputs(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut original_fn: ItemFn = parse_macro_input!(item as ItemFn);

    let attr = parse_macro_input!(attr as AttributeArgs);
    let parsed_attributes = match InputOptions::from_list(&attr) {
        Ok(val) => val,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    let mut stmts = match log_fn_inputs(&original_fn, parsed_attributes) {
        Ok(input_log) => vec![input_log],
        Err(e) => return e.to_compile_error().into(),
    };

    stmts.extend(original_fn.block.stmts);
    original_fn.block.stmts = stmts;
    original_fn.into_token_stream().into()
}

fn log_fn_inputs(func: &ItemFn, attr: InputOptions) -> syn::Result<Stmt> {
    let fn_name = func.sig.ident.to_string();
    let inputs: Vec<Ident> = func
        .sig
        .inputs
        .iter()
        .cloned()
        .map(|arg| match arg {
            FnArg::Receiver(arg) => arg.self_token.into(),
            FnArg::Typed(pat_type) => {
                if let Pat::Ident(ident) = *pat_type.pat {
                    ident.ident
                } else {
                    unimplemented!()
                }
            }
        })
        .collect();

    let items: Punctuated<_, token::Comma> = inputs.iter().cloned().collect();

    let level = get_logger_token(&attr.level);
    let fmt = attr.fmt.unwrap_or_else(|| {
        let mut fmt = String::with_capacity(inputs.len() * 9);
        fmt.push_str(&fn_name);
        fmt.push('(');

        for input in inputs {
            fmt.push_str(&input.to_string());
            fmt.push_str(": {:?},");
        }
        fmt.pop(); // Remove the extra comma.
        fmt.push(')');
        fmt
    });

    let res = quote! {
        log::log!(#level, #fmt, #items);
    };
    syn::parse2(res)
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::is_result_type;

    #[test]
    fn result_type() {
        assert!(is_result_type(&parse_quote!(Result<T, E>)));
        assert!(is_result_type(&parse_quote!(std::result::Result<T, E>)));
        assert!(is_result_type(&parse_quote!(fmt::Result)));
    }
}
