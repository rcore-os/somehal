use proc_macro::TokenStream;
use quote::{format_ident, quote};

mod print;

#[proc_macro]
pub fn println(input: TokenStream) -> TokenStream {
    match print::println(input) {
        Ok(o) => o,
        Err(e) => e.into_compile_error().into(),
    }
}

#[proc_macro]
pub fn fn_link_section(input: TokenStream) -> TokenStream {
    let name = input.to_string();

    let id_start = format_ident!("__start_{name}");
    let id_stop = format_ident!("__stop_{name}");

    let id_fn = format_ident!("{name}");

    quote! {
        #[inline(always)]
        fn #id_fn() -> &'static [u8] {
            unsafe extern "C" {
                fn #id_start();
                fn #id_stop();
            }

            unsafe {
                let start = #id_start as usize;
                let stop = #id_stop as usize;
                &core::slice::from_raw_parts(start as *const u8, stop - start)
            }
        }
    }
    .into()
}
