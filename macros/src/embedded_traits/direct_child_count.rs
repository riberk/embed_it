use embed_it_utils::entry::EntryKind;
use quote::quote;
use syn::parse_quote;

use crate::{
    embed::{attributes::embed::GenerationSettings, EntryTokens, GenerateContext, IndexTokens},
    embedded_traits::MakeEmbeddedTraitImplementationError,
};

use super::EmbeddedTrait;

#[derive(Debug)]
pub struct DirectChildCountTrait;

fn method() -> syn::Ident {
    parse_quote!(direct_child_count)
}

impl EmbeddedTrait for DirectChildCountTrait {
    fn path(&self, _nesting: usize, _: &GenerationSettings) -> syn::Path {
        parse_quote!(::embed_it::DirectChildCount)
    }

    fn impl_body(
        &self,
        ctx: &mut GenerateContext<'_>,
        entries: &[EntryTokens],
        _index: &[IndexTokens],
    ) -> Result<proc_macro2::TokenStream, MakeEmbeddedTraitImplementationError> {
        if ctx.entry.kind() != EntryKind::Dir {
            return Err(MakeEmbeddedTraitImplementationError::UnsupportedEntry {
                entry: ctx.entry.kind(),
                trait_id: self.id(),
            });
        }
        let method = method();
        let len = entries.len();
        Ok(quote! {
            fn #method(&self) -> usize {
                #len
            }
        })
    }

    fn definition(&self, _: &GenerationSettings) -> Option<proc_macro2::TokenStream> {
        None
    }

    fn id(&self) -> &'static str {
        "DirectChildCount"
    }
}
