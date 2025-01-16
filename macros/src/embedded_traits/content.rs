use quote::quote;
use syn::parse_quote;

use crate::{
    embed::{EntryTokens, GenerateContext, IndexTokens},
    embedded_traits::EmbeddedTrait,
    fs::EntryKind,
};

pub struct ContentTrait;

impl EmbeddedTrait for ContentTrait {
    fn path(&self, _nesting: usize) -> syn::Path {
        parse_quote!(::embed_it::Content)
    }

    fn impl_body(
        &self,
        ctx: &GenerateContext<'_>,
        _entries: &[EntryTokens],
        _index: &[IndexTokens],
    ) -> proc_macro2::TokenStream {
        if ctx.entry.kind() != EntryKind::File {
            panic!(
                "Only files are supported to derive '{:?}'",
                self.path(ctx.level)
            )
        }
        let origin = &ctx.entry.path().origin;
        quote! {
            fn content(&self) -> &'static [u8] {
                const VALUE: &[u8] = include_bytes!(#origin);
                VALUE
            }
        }
    }

    fn definition(&self, _: &syn::Ident) -> Option<proc_macro2::TokenStream> {
        None
    }

    fn id(&self) -> &'static str {
        "Content"
    }

    fn entry_impl_body(&self) -> proc_macro2::TokenStream {
        panic!("Only files are supported to derive '{:?}'", self.path(0));
    }
}
