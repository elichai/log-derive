use quote::quote;

use syn::{ExprClosure, ItemFn, Result};

use super::FormattedAttributes;

pub(crate) fn generate_function(_f: &ItemFn, closure: &ExprClosure, expressions: FormattedAttributes, result: bool) -> Result<ItemFn> {
    let FormattedAttributes { ok_expr, err_expr, log_ts, contained_ok_or_err } = expressions;
    let result = result || contained_ok_or_err;
    let code = if log_ts {
        if result {
            quote! {
                fn temp() {
                    let instant = std::time::Instant::now();
                    let result = (#closure)();
                    let ts = instant.elapsed();
                    result.map(|result| { #ok_expr; result })
                        .map_err(|err| { #err_expr; err })
                }
            }
        } else {
            quote! {
                fn temp() {
                    let instant = std::time::Instant::now();
                    let result = (#closure)();
                    let ts = instant.elapsed();
                    #ok_expr;
                    result
                }
            }
        }
    } else if result {
        quote! {
            fn temp() {
                let result = (#closure)();
                result.map(|result| { #ok_expr; result })
                    .map_err(|err| { #err_expr; err })
            }
        }
    } else {
        quote! {
            fn temp() {
                let result = (#closure)();
                #ok_expr;
                result
            }
        }
    };

    syn::parse2(code)
}
