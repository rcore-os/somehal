use darling::{FromMeta, ast::NestedMeta};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, parse_macro_input};

mod arch;
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

/// A speaking volume. Deriving `FromMeta` will cause this to be usable
/// as a string value for a meta-item key.
#[derive(Debug, Clone, Copy, FromMeta)]
#[darling(default)]
enum Aarch64TrapHandlerKind {
    Irq,
    Fiq,
    Sync,
    #[darling(rename = "serror")]
    SError,
}

#[derive(Debug, FromMeta)]
struct Aarch64TrapHandlerArgs {
    kind: Aarch64TrapHandlerKind,
}

#[proc_macro_attribute]
pub fn aarch64_trap_handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(darling::Error::from(e).write_errors());
        }
    };
    let args = match Aarch64TrapHandlerArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let func = parse_macro_input!(input as ItemFn);

    match args.kind {
        Aarch64TrapHandlerKind::Irq | Aarch64TrapHandlerKind::Fiq => {
            arch::aarch64::trap_handle_irq(func).into()
        }
        Aarch64TrapHandlerKind::Sync => arch::aarch64::trap_handle_irq(func).into(),
        Aarch64TrapHandlerKind::SError => arch::aarch64::trap_handle_irq(func).into(),
    }
}
