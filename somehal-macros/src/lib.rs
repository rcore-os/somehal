use proc_macro::TokenStream;

mod print;

#[proc_macro]
pub fn println(input: TokenStream) -> TokenStream {
    match print::println(input) {
        Ok(o) => o,
        Err(e) => e.into_compile_error().into(),
    }
}
