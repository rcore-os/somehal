use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, Expr, Lit, Token, parse::Parser, punctuated::Punctuated, spanned::Spanned};
pub fn println(input: TokenStream) -> Result<TokenStream, Error> {
    let tokens = input.clone();
    let parser = Punctuated::<Expr, Token![,]>::parse_terminated;
    let args = parser.parse(tokens)?;
    // let mut items = Vec::new();

    let format_str;

    let mut args = args.into_iter();

    if let Some(fmt) = args.next() {
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

    println!("fmt: {}", format_str);

    let mut right = format_str;

    while let Some(pat) = find_patterns(&right) {
        
        println!("{:?}", pat);

        

        right = pat.right;
    }

    Ok(quote! {}.into())
}
#[derive(Debug)]
struct Pattern {
    left: String,
    pattern: String,
    right: String,
}

fn find_patterns(format_str: &str) -> Option<Pattern> {
    let patterns = ["{}", "{:#x}"];
    let mut i: Option<usize> = None;
    let mut out = None;

    for pat in patterns {
        if let Some(n) = format_str.find(pat) {
            if let Some(l) = i {
                if n > l {
                    continue;
                }
            }
            i = Some(n);
            let left = format_str[..n].to_string();
            let right = format_str[n..].trim_start_matches(pat).to_string();
            out = Some(Pattern {
                left,
                pattern: pat.to_string(),
                right,
            });
            
        }
    }

    out
}

// fn split_format_string(format_str: &str) -> Vec<String> {
//     let patterns = ["{}", "{:#x}"];
//     let mut result = Vec::new();
//     let mut start = 0;
//     let mut end = 0;
//     let mut in_placeholder = false;

//     while end < format_str.len() {
//         if format_str.get(end..end + 2) == Some("{}") {
//             if !in_placeholder {
//                 result.push(&format_str[start..end]);
//             }
//         }
//     }
// }
