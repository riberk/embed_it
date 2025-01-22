use quote::quote;
use syn::parse_quote;

use crate::{
    embed::{EntryTokens, GenerateContext, IndexTokens},
    embedded_traits::MakeEmbeddedTraitImplementationError,
    fs::EntryKind,
};

use super::EmbeddedTrait;

#[derive(Debug)]
pub struct RecursiveChildCountTrait;

fn method() -> syn::Ident {
    parse_quote!(recursive_child_count)
}

impl EmbeddedTrait for RecursiveChildCountTrait {
    fn path(&self, _nesting: usize) -> syn::Path {
        parse_quote!(::embed_it::RecursiveChildCount)
    }

    fn impl_body(
        &self,
        ctx: &mut GenerateContext<'_>,
        _entries: &[EntryTokens],
        index: &[IndexTokens],
    ) -> Result<proc_macro2::TokenStream, MakeEmbeddedTraitImplementationError> {
        if ctx.entry.kind() != EntryKind::Dir {
            return Err(MakeEmbeddedTraitImplementationError::UnsupportedEntry {
                entry: ctx.entry.kind(),
                trait_id: self.id(),
            });
        }
        let method = method();
        let len = index.len();
        Ok(quote! {
            fn #method(&self) -> usize {
                #len
            }
        })
    }

    fn definition(&self, _: &syn::Ident) -> Option<proc_macro2::TokenStream> {
        None
    }

    fn id(&self) -> &'static str {
        "RecursiveChildCount"
    }

    fn entry_impl_body(&self) -> proc_macro2::TokenStream {
        panic!("Only dirs are supported to derive '{:?}'", self.id())
    }
}
