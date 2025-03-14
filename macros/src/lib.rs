use embed::impl_embed;
use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

pub(crate) mod embed;
pub(crate) mod embedded_traits;
pub(crate) mod fs;
pub(crate) mod main_trait_data;
pub(crate) mod marker_traits;
pub(crate) mod utils;

#[cfg(test)]
pub(crate) mod test_helpers;

#[proc_macro_derive(Embed, attributes(embed))]
pub fn derive_embed(input: TokenStream) -> TokenStream {
    match impl_embed(parse_macro_input!(input as DeriveInput)) {
        Ok(token_stream) => token_stream.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
