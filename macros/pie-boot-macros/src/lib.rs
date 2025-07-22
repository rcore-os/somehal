use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, ItemFn, parse::Parse, parse_macro_input};

mod entry;

#[proc_macro_attribute]
pub fn start_code(args: TokenStream, input: TokenStream) -> TokenStream {
    // 解析参数中的选项，例如 naked
    let parsed_args = parse_macro_input!(args as StartCodeArgs);

    let f = parse_macro_input!(input as ItemFn);
    let attrs = f.attrs;
    let vis = f.vis;
    let name = f.sig.ident;
    let args = f.sig.inputs;
    let stmts = f.block.stmts;
    let ret = f.sig.output;

    let naked_prefix;
    let naked_attr;
    if parsed_args.naked {
        naked_attr = quote! {
            #[unsafe(naked)]
        };
        naked_prefix = quote! {
            unsafe extern "C"
        };
    } else {
        naked_attr = quote! {};
        naked_prefix = quote! {};
    };

    quote!(
        #naked_attr
        #[unsafe(link_section = ".idmap.text")]
        #(#attrs)*
        #vis #naked_prefix fn #name(#args) #ret {
            #(#stmts)*
        }
    )
    .into()
}

struct StartCodeArgs {
    naked: bool,
}

impl Parse for StartCodeArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut naked = false;

        if !input.is_empty() {
            let ident: Ident = input.parse()?;
            if ident == "naked" {
                naked = true;
            } else {
                return Err(input.error("unexpected argument, expected `naked`"));
            }
        }

        Ok(StartCodeArgs { naked })
    }
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
/// # use pie_boot::entry;
/// #[entry]
/// fn main(args: &pie_boot::BootArgs) -> ! {
///     loop {
///         /* .. */
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::entry(args, input, "__pie_boot_main")
}

/// Attribute to declare the secondary entry point of the program
///
/// # Examples
///
/// - Simple entry point
///
/// ``` no_run
/// # #![no_main]
/// # use pie_boot::secondary_entry;
/// #[entry]
/// fn secondary(cpu_id: usize) -> ! {
///     loop {
///         /* .. */
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn secondary_entry(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::entry(args, input, "__pie_boot_secondary")
}
