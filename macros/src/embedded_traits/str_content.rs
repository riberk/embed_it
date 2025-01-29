use embed_it_utils::entry::EntryKind;
use quote::quote;
use syn::parse_quote;

use crate::{
    embed::{attributes::embed::GenerationSettings, EntryTokens, GenerateContext, IndexTokens},
    embedded_traits::EmbeddedTrait,
};

use super::MakeEmbeddedTraitImplementationError;

#[derive(Debug)]
pub struct StrContentTrait;

impl EmbeddedTrait for StrContentTrait {
    fn path(&self, _nesting: usize, _: &GenerationSettings) -> syn::Path {
        parse_quote!(::embed_it::StrContent)
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
            fn str_content(&self) -> &'static str {
                const VALUE: &str = include_str!(#origin);
                VALUE
            }
        })
    }

    fn definition(&self, _: &GenerationSettings) -> Option<proc_macro2::TokenStream> {
        None
    }

    fn id(&self) -> &'static str {
        "StrContent"
    }
}
