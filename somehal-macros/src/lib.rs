use darling::{FromMeta, ast::NestedMeta};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    FnArg, ItemFn, PathArguments, Type, Visibility, parse, parse_macro_input, spanned::Spanned,
};

use crate::entry::entry;

mod arch;
mod entry;
mod print;

/// Attribute to declare the entry point of the program
///
/// **IMPORTANT**: This attribute must appear exactly *once* in the dependency graph. Also, if you
/// are using Rust 1.30 the attribute must be used on a reachable item (i.e. there must be no
/// private modules between the item and the root of the crate); if the item is in the root of the
/// crate you'll be fine. This reachability restriction doesn't apply to Rust 1.31 and newer releases.
///
/// The specified function will be called by the reset handler *after* RAM has been initialized.
/// If present, the FPU will also be enabled before the function is called.
///
/// The type of the specified function must be `[unsafe] fn() -> !` (never ending function)
///
/// # Properties
///
/// The entry point will be called by the reset handler. The program can't reference to the entry
/// point, much less invoke it.
///
/// # Examples
///
/// - Simple entry point
///
/// ``` no_run
/// # #![no_main]
/// # use sparreal_macros::entry;
/// #[entry]
/// fn main() -> ! {
///     loop {
///         /* .. */
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn entry_hal(args: TokenStream, input: TokenStream) -> TokenStream {
    entry(args, input, "__somehal_main")
}

/// Attribute to declare the entry point of the program
///
/// **IMPORTANT**: This attribute must appear exactly *once* in the dependency graph. Also, if you
/// are using Rust 1.30 the attribute must be used on a reachable item (i.e. there must be no
/// private modules between the item and the root of the crate); if the item is in the root of the
/// crate you'll be fine. This reachability restriction doesn't apply to Rust 1.31 and newer releases.
///
/// The specified function will be called by the reset handler *after* RAM has been initialized.
/// If present, the FPU will also be enabled before the function is called.
///
/// The type of the specified function must be `[unsafe] fn() -> !` (never ending function)
///
/// # Properties
///
/// The entry point will be called by the reset handler. The program can't reference to the entry
/// point, much less invoke it.
///
/// # Examples
///
/// - Simple entry point
///
/// ``` no_run
/// # #![no_main]
/// # use somert::entry;
/// #[entry]
/// fn main() -> ! {
///     loop {
///         /* .. */
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn entry_rt(args: TokenStream, input: TokenStream) -> TokenStream {
    entry(args, input, "__somert_main")
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

#[proc_macro]
pub fn dbgln(input: TokenStream) -> TokenStream {
    match print::dbgln(input) {
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
                core::slice::from_raw_parts(start as *const u8, stop - start)
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
