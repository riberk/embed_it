use embed_it_utils::entry::EntryKind;
use quote::quote;
use syn::parse_quote;

use crate::{
    embed::{EntryTokens, GenerateContext, IndexTokens, attributes::embed::GenerationSettings},
    embedded_traits::EmbeddedTrait,
};

use super::MakeEmbeddedTraitImplementationError;

#[derive(Debug)]
pub struct ContentTrait;

impl EmbeddedTrait for ContentTrait {
    fn path(&self, _nesting: usize, _: &GenerationSettings) -> syn::Path {
        parse_quote!(::embed_it::Content)
    }

    fn impl_body(
        &self,
        ctx: &mut GenerateContext<'_>,
        _entries: &[EntryTokens],
        _index: &[IndexTokens],
    ) -> Result<proc_macro2::TokenStream, MakeEmbeddedTraitImplementationError> {
        if ctx.entry.kind() != EntryKind::File {
            return Err(MakeEmbeddedTraitImplementationError::UnsupportedEntry {
                entry: ctx.entry.kind(),
                trait_id: self.id(),
            });
        }

        let origin = &ctx.entry.as_ref().value().path().origin;
        Ok(quote! {
            fn content(&self) -> &'static [u8] {
                const VALUE: &[u8] = include_bytes!(#origin);
                VALUE
            }
        })
    }

    fn definition(&self, _: &GenerationSettings) -> Option<proc_macro2::TokenStream> {
        None
    }

    fn id(&self) -> &'static str {
        "Content"
    }
}
