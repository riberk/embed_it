use quote::quote;
use syn::parse_quote;

use crate::{
    embed::{EntryTokens, GenerateContext, IndexTokens},
    fs::EntryPath,
};

use super::EmbeddedTrait;

pub struct PathTrait;

fn method() -> syn::Ident {
    parse_quote!(path)
}

impl EmbeddedTrait for PathTrait {
    fn path(&self, _nesting: usize) -> syn::Path {
        parse_quote!(::embed_it::EntryPath)
    }

    fn impl_body(
        &self,
        ctx: &GenerateContext<'_>,
        _entries: &[EntryTokens],
        _index: &[IndexTokens],
    ) -> proc_macro2::TokenStream {
        let EntryPath {
            relative: relative_path,
            file_name,
            file_stem,
            ..
        } = ctx.entry.path();

        let method = method();
        quote! {
            fn #method(&self) -> &'static ::embed_it::EmbeddedPath {
                const VALUE: &::embed_it::EmbeddedPath = &::embed_it::EmbeddedPath::new(#relative_path, #file_name, #file_stem);
                VALUE
            }
        }
    }

    fn definition(&self, _: &syn::Ident) -> Option<proc_macro2::TokenStream> {
        None
    }

    fn id(&self) -> &'static str {
        "Path"
    }

    fn entry_impl_body(&self) -> proc_macro2::TokenStream {
        let method = method();
        quote! {
            fn #method(&self) -> &'static ::embed_it::EmbeddedPath {
                match self {
                    Self::Dir(d) => d.#method(),
                    Self::File(f) => f.#method(),
                }
            }
        }
    }
}
