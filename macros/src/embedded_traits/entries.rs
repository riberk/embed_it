use quote::quote;
use syn::parse_quote;

use crate::{
    embed::{EntryTokens, GenerateContext, IndexTokens},
    embedded_traits::MakeEmbeddedTraitImplementationError,
    fs::EntryKind,
};

use super::EmbeddedTrait;

#[derive(Debug)]
pub struct EntriesTrait;

fn ident() -> syn::Ident {
    parse_quote!(Entries)
}

fn method() -> syn::Ident {
    parse_quote!(entries)
}

impl EmbeddedTrait for EntriesTrait {
    fn path(&self, nesting: usize) -> syn::Path {
        GenerateContext::make_nested_path(nesting, ident())
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
        let entry_path = &ctx.entry_path;
        let method = method();
        let entries = entries.iter().fold(quote! {}, |mut entries, tokens| {
            let EntryTokens {
                struct_path, entry, ..
            } = tokens;
            let kind_ident = entry.kind().ident();
            entries.extend(quote! {
                #entry_path::#kind_ident(&#struct_path),
            });

            entries
        });

        Ok(quote! {
            fn #method(&self) -> &'static [#entry_path] {
                const VALUE: &[#entry_path] = &[
                    #entries
                ];
                VALUE
            }
        })
    }

    fn definition(&self, entry_path: &syn::Ident) -> Option<proc_macro2::TokenStream> {
        let ident = ident();
        let method = method();
        Some(quote! {
            pub trait #ident {
                fn #method(&self) -> &'static [#entry_path];
            }
        })
    }

    fn id(&self) -> &'static str {
        "Entries"
    }

    fn entry_impl_body(&self) -> proc_macro2::TokenStream {
        panic!("Only dirs are supported to derive '{:?}'", ident())
    }
}
