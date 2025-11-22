use quote::quote;
use syn::parse_quote;

use crate::{
    embed::{EntryTokens, GenerateContext, IndexTokens, attributes::embed::GenerationSettings},
    fs::EntryPath,
};

use super::{EmbeddedTrait, MakeEmbeddedTraitImplementationError};

#[derive(Debug)]
pub struct PathTrait;

fn method() -> syn::Ident {
    parse_quote!(path)
}

impl PathTrait {
    fn impl_body(
        &self,
        ctx: &mut GenerateContext<'_>,
    ) -> Result<proc_macro2::TokenStream, MakeEmbeddedTraitImplementationError> {
        let EntryPath {
            relative: relative_path,
            file_name,
            file_stem,
            ..
        } = ctx.entry.as_ref().value().path();

        let method = method();
        Ok(quote! {
            pub fn #method(&self) -> &'static ::embed_it::EmbeddedPath {
                const VALUE: &::embed_it::EmbeddedPath = &::embed_it::EmbeddedPath::new(#relative_path, #file_name, #file_stem);
                VALUE
            }
        })
    }
}

impl EmbeddedTrait for PathTrait {
    fn path(&self, _nesting: usize, _: &GenerationSettings) -> syn::Path {
        parse_quote!(::embed_it::EntryPath)
    }

    fn impl_body(
        &self,
        ctx: &mut GenerateContext<'_>,
        _entries: &[EntryTokens],
        _index: &[IndexTokens],
    ) -> Option<Result<proc_macro2::TokenStream, MakeEmbeddedTraitImplementationError>> {
        Some(self.impl_body(ctx))
    }

    fn definition(&self, _: &GenerationSettings) -> Option<proc_macro2::TokenStream> {
        None
    }

    fn id(&self) -> &'static str {
        "Path"
    }
    fn impl_trait_body(
        &self,
        _ctx: &mut GenerateContext<'_>,
        _entries: &[EntryTokens],
        _index: &[IndexTokens],
    ) -> Result<proc_macro2::TokenStream, MakeEmbeddedTraitImplementationError> {
        let method = method();
        Ok(quote! {
            fn #method(&self) -> &'static ::embed_it::EmbeddedPath {
                self.#method()
            }
        })
    }
}
