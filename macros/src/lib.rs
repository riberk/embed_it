use embedded_dir::impl_assets;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod embedded_dir;

#[proc_macro_derive(Assets, attributes(assets))]
pub fn derive_assets(input: TokenStream) -> TokenStream {
    match impl_assets(parse_macro_input!(input as DeriveInput)) {
        Ok(token_stream) => token_stream.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
