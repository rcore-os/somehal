use darling::{FromMeta, ast::NestedMeta};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    FnArg, ItemFn, PathArguments, Type, Visibility, parse, parse_macro_input, spanned::Spanned,
};

pub fn entry(args: TokenStream, input: TokenStream, name: &str) -> TokenStream {
    let f = parse_macro_input!(input as ItemFn);

    // check the function arguments
    if f.sig.inputs.len() > 3 {
        return parse::Error::new(
            f.sig.inputs.last().unwrap().span(),
            "`#[entry]` function has too many arguments",
        )
        .to_compile_error()
        .into();
    }
    // for arg in &f.sig.inputs {
    //     match arg {
    //         FnArg::Receiver(_) => {
    //             return parse::Error::new(arg.span(), "invalid argument")
    //                 .to_compile_error()
    //                 .into();
    //         }
    //         FnArg::Typed(t) => {
    //             if !is_simple_type(&t.ty, "usize") {
    //                 return parse::Error::new(t.ty.span(), "argument type must be usize")
    //                     .to_compile_error()
    //                     .into();
    //             }
    //         }
    //     }
    // }

    // check the function signature
    let valid_signature = f.sig.constness.is_none()
        && f.sig.asyncness.is_none()
        && f.vis == Visibility::Inherited
        && f.sig.abi.is_none()
        && f.sig.generics.params.is_empty()
        && f.sig.generics.where_clause.is_none()
        && f.sig.variadic.is_none()
        // && match f.sig.output {
        //     ReturnType::Default => false,
        //     ReturnType::Type(_, ref ty) => matches!(**ty, Type::Never(_)),
        // }
        ;

    if !valid_signature {
        return parse::Error::new(
            f.span(),
            "`#[entry]` function must have signature `[unsafe] fn([arg0: usize, ...]) `",
        )
        .to_compile_error()
        .into();
    }

    if !args.is_empty() {
        return parse::Error::new(Span::call_site(), "This attribute accepts no arguments")
            .to_compile_error()
            .into();
    }

    // XXX should we blacklist other attributes?
    let attrs = f.attrs;
    let unsafety = f.sig.unsafety;
    let args = f.sig.inputs;
    let stmts = f.block.stmts;
    let name = format_ident!("{}", name);

    quote!(
        #[allow(non_snake_case)]
        #[unsafe(no_mangle)]
        #(#attrs)*
        pub #unsafety extern "Rust" fn #name(#args) {
            #(#stmts)*
        }
    )
    .into()
}

#[allow(unused)]
fn is_simple_type(ty: &Type, name: &str) -> bool {
    if let Type::Path(p) = ty
        && p.qself.is_none()
        && p.path.leading_colon.is_none()
        && p.path.segments.len() == 1
    {
        let segment = p.path.segments.first().unwrap();
        if segment.ident == name && segment.arguments == PathArguments::None {
            return true;
        }
    }
    false
}
