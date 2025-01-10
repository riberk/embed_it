use embedded_dir::impl_embedded_dir;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod embedded_dir;

#[proc_macro_derive(EmbeddedDir, attributes(embedded_dir))]
pub fn derive_embedded_dir(input: TokenStream) -> TokenStream {
    match impl_embedded_dir(parse_macro_input!(input as DeriveInput)) {
        Ok(token_stream) => token_stream.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
