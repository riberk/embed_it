use embed_it_utils::entry::EntryKind;
use quote::quote;
use syn::parse_quote;

use crate::{
    embed::{EntryTokens, GenerateContext, IndexTokens, attributes::embed::GenerationSettings},
    embedded_traits::MakeEmbeddedTraitImplementationError,
    utils::entry_ext::EntryKindExt,
};

use super::EmbeddedTrait;

#[derive(Debug)]
pub struct EntriesTrait;

impl EmbeddedTrait for EntriesTrait {
    fn path(&self, level: usize, settings: &GenerationSettings) -> syn::Path {
        let dir = settings.dir_entry_param(level);
        let file = settings.file_entry_param(level);
        parse_quote!(::embed_it::Entries<#dir, #file>)
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
        let entry_path = &ctx.settings.entry_path(ctx.level);
        let entries = entries.iter().fold(quote! {}, |mut entries, tokens| {
            let EntryTokens {
                struct_path, entry, ..
            } = tokens;
            let kind_ident = entry.kind().ident();
            let entry_struct_path = ctx.settings.entry_param_for(entry.kind(), ctx.level);
            entries.extend(quote! {
                ::embed_it::Entry::#kind_ident(#entry_struct_path(&#struct_path)),
            });

            entries
        });

        Ok(quote! {
            fn entries(&self) -> &'static [#entry_path] {
                const VALUE: &[#entry_path] = &[
                    #entries
                ];
                VALUE
            }
        })
    }

    fn definition(&self, _: &GenerationSettings) -> Option<proc_macro2::TokenStream> {
        None
    }

    fn id(&self) -> &'static str {
        "Entries"
    }
}
