use quote::quote;

use syn::{ExprClosure, ItemFn, Result};

use super::FormattedAttributes;

pub(crate) fn generate_function(f: &ItemFn, closure: &ExprClosure, expressions: FormattedAttributes, result: bool) -> Result<ItemFn> {
    if f.sig.asyncness.is_none() {
        return super::generate_function::generate_function(f, closure, expressions, result);
    }
    let FormattedAttributes { ok_expr, err_expr, log_ts, contained_ok_or_err } = expressions;
    let result = result || contained_ok_or_err;
    let code = if log_ts {
        if result {
            quote! {
                async fn temp() {
                    let instant = std::time::Instant::now();
                    let result = (#closure)().await;
                    let ts = instant.elapsed();
                    result.map(|result| { #ok_expr; result })
                        .map_err(|err| { #err_expr; err })
                }
            }
        } else {
            quote! {
                async fn temp() {
                    let instant = std::time::Instant::now();
                    let result = (#closure)().await;
                    let ts = instant.elapsed();
                    #ok_expr;
                    result
                }
            }
        }
    } else if result {
        quote! {
            async fn temp() {
                let result = (#closure)().await;
                result.map(|result| { #ok_expr; result })
                    .map_err(|err| { #err_expr; err })
            }
        }
    } else {
        quote! {
            async fn temp() {
                let result = (#closure)().await;
                #ok_expr;
                result
            }
        }
    };

    syn::parse2(code)
}
