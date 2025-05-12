use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStatic, parse_macro_input};

#[proc_macro_attribute]
pub fn def_percpu(_args: TokenStream, input: TokenStream) -> TokenStream {
    let ItemStatic {
        attrs,
        vis,
        static_token,
        mutability,
        ident,
        ty,
        expr,
        ..
    } = parse_macro_input!(input as ItemStatic);

    quote! {
        #[unsafe(link_section = ".data.percpu")]
        #(#attrs)*
        #vis #static_token #mutability #ident : kpercpu::PerCpuData<#ty> = kpercpu::PerCpuData::new(#expr);
    }
    .into()
}
