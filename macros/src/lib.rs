use embed::impl_embed;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod embed;

#[proc_macro_derive(Embed, attributes(embed))]
pub fn derive_embed(input: TokenStream) -> TokenStream {
    match impl_embed(parse_macro_input!(input as DeriveInput)) {
        Ok(token_stream) => token_stream.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
