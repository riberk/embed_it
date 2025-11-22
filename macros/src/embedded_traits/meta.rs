use std::time::{Duration, SystemTime};

use quote::quote;
use syn::parse_quote;

use crate::embed::{
    EntryTokens, GenerateContext, IndexTokens, attributes::embed::GenerationSettings,
};

use super::{EmbeddedTrait, MakeEmbeddedTraitImplementationError};

#[derive(Debug)]
pub struct MetaTrait;

fn method() -> syn::Ident {
    parse_quote!(metadata)
}

impl MetaTrait {
    fn impl_body(
        &self,
        ctx: &mut GenerateContext<'_>,
    ) -> Result<proc_macro2::TokenStream, MakeEmbeddedTraitImplementationError> {
        fn unixtime(t: SystemTime) -> Duration {
            t.duration_since(SystemTime::UNIX_EPOCH).unwrap()
        }

        fn constructor(t: SystemTime) -> proc_macro2::TokenStream {
            let duration = unixtime(t);
            let secs = duration.as_secs();
            let nanos = duration.subsec_nanos();
            quote! {
                Some(std::time::Duration::new(#secs, #nanos))
            }
        }

        fn make_stream(value: std::io::Result<SystemTime>) -> proc_macro2::TokenStream {
            value.ok().map(constructor).unwrap_or_else(|| quote! {None})
        }

        let meta = &ctx.entry.as_ref().value().metadata();

        let accessed = make_stream(meta.accessed());
        let created = make_stream(meta.created());
        let modified = make_stream(meta.modified());

        let method = method();
        Ok(quote! {
            pub fn #method(&self) -> &'static ::embed_it::Metadata {
                const VALUE: &::embed_it::Metadata = &::embed_it::Metadata::new(
                    #accessed,
                    #created,
                    #modified,
                );
                VALUE
            }
        })
    }
}

impl EmbeddedTrait for MetaTrait {
    fn path(&self, _nesting: usize, _: &GenerationSettings) -> syn::Path {
        parse_quote!(::embed_it::Meta)
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
        "Meta"
    }

    fn impl_trait_body(
        &self,
        _ctx: &mut GenerateContext<'_>,
        _entries: &[EntryTokens],
        _index: &[IndexTokens],
    ) -> Result<proc_macro2::TokenStream, MakeEmbeddedTraitImplementationError> {
        let method = method();
        Ok(quote! {
            fn #method(&self) -> &'static ::embed_it::Metadata {
                self.#method()
            }
        })
    }
}
