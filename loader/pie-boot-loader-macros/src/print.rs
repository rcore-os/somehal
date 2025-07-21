use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, Expr, Lit, Token, parse::Parser, punctuated::Punctuated, spanned::Spanned};

pub fn println(input: TokenStream) -> Result<TokenStream, Error> {
    let tokens = input.clone();
    let parser = Punctuated::<Expr, Token![,]>::parse_terminated;
    let args = parser.parse(tokens)?;
    let arg_count = args.len() - 1;

    let format_str;

    let mut args_iter = args.clone().into_iter();

    if let Some(fmt) = args_iter.next() {
        if let Expr::Lit(expr_lit) = fmt {
            if let Lit::Str(lit_str) = expr_lit.lit {
                // 第一个参数是字符串字面量
                format_str = lit_str.value();
            } else {
                return Err(Error::new_spanned(
                    expr_lit.lit,
                    "Expected a string literal",
                ));
            }
        } else {
            return Err(Error::new(
                fmt.span(),
                "println! macro only accept string as fmt",
            ));
        }
    } else {
        return Ok(quote! {
            crate::console::__print_str("\r\n");
        }
        .into());
    }

    let mut right = format_str;
    let mut items = Vec::new();

    if arg_count == 0 {
        items.push(quote! {
           crate::console::__print_str(#right);
        });
    } else {
        while let Some(pat) = find_patterns(&right) {
            let left = pat.left;

            items.push(quote! {
                crate::console::__print_str(#left);
            });

            let arg = args_iter
                .next()
                .ok_or(Error::new(args.span(), "args not match fmt"))?;

            items.push(quote! {
                {
                    crate::console::Print::_print(#arg);
                }
            });

            right = pat.right;
        }
        if !right.is_empty() {
            items.push(quote! {
                crate::console::__print_str(#right);
            });
        }
    }

    Ok(quote! {
        {
            #(#items);*
            crate::console::__print_str("\r\n");
        }
    }
    .into())
}

#[derive(Debug)]
struct Pattern {
    left: String,
    _pattern: String,
    right: String,
}

fn find_patterns(format_str: &str) -> Option<Pattern> {
    let patterns = ["{}"];
    let mut i: Option<usize> = None;
    let mut out = None;

    for pat in patterns {
        if let Some(n) = format_str.find(pat) {
            if let Some(l) = i
                && n > l
            {
                continue;
            }
            i = Some(n);
            let left = format_str[..n].to_string();
            let right = format_str[n..].trim_start_matches(pat).to_string();
            out = Some(Pattern {
                left,
                _pattern: pat.to_string(),
                right,
            });
        }
    }

    out
}
