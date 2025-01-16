use std::time::{Duration, SystemTime};

use quote::quote;
use syn::parse_quote;

use crate::embed::{EntryTokens, GenerateContext, IndexTokens};

use super::EmbeddedTrait;

pub struct MetaTrait;

fn method() -> syn::Ident {
    parse_quote!(metadata)
}

impl EmbeddedTrait for MetaTrait {
    fn path(&self, _nesting: usize) -> syn::Path {
        parse_quote!(::embed_it::Meta)
    }

    fn impl_body(
        &self,
        ctx: &GenerateContext<'_>,
        _entries: &[EntryTokens],
        _index: &[IndexTokens],
    ) -> proc_macro2::TokenStream {
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

        let meta = &ctx.entry.metadata();

        let accessed = make_stream(meta.accessed());
        let created = make_stream(meta.created());
        let modified = make_stream(meta.modified());

        let method = method();
        quote! {
            fn #method(&self) -> &'static ::embed_it::Metadata {
                const VALUE: &::embed_it::Metadata = &::embed_it::Metadata::new(
                    #accessed,
                    #created,
                    #modified,
                );
                VALUE
            }
        }
    }

    fn definition(&self, _: &syn::Ident) -> Option<proc_macro2::TokenStream> {
        None
    }

    fn id(&self) -> &'static str {
        "Meta"
    }

    fn entry_impl_body(&self) -> proc_macro2::TokenStream {
        let method = method();
        quote! {
            fn #method(&self) -> &'static ::embed_it::Metadata {
                match self {
                    Self::Dir(d) => d.#method(),
                    Self::File(f) => f.#method(),
                }
            }
        }
    }
}
