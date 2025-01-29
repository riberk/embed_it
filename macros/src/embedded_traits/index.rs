use embed_it_utils::entry::EntryKind;
use quote::quote;
use syn::parse_quote;

use crate::{
    embed::{
        attributes::embed::GenerationSettings, bool_like_enum::BoolLikeEnum, EntryTokens,
        GenerateContext, IndexTokens,
    },
    embedded_traits::MakeEmbeddedTraitImplementationError,
    utils::entry_ext::EntryKindExt,
};

use super::EmbeddedTrait;

#[derive(Debug)]
pub struct IndexTrait;

impl EmbeddedTrait for IndexTrait {
    fn path(&self, level: usize, settings: &GenerationSettings) -> syn::Path {
        let dir = settings.dir_entry_param(level);
        let file = settings.file_entry_param(level);
        parse_quote!(::embed_it::Index<#dir, #file>)
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
        let entry_path = &ctx.settings.entry_path(ctx.level);
        let index_len = index.len();

        let struct_ident = &ctx.struct_ident;
        let entry_struct_path = ctx.settings.dir_entry_param(ctx.level);
        let index = index
            .iter()
            .fold(quote! {
                map.insert("", ::embed_it::Entry::Dir(#entry_struct_path(&#struct_ident)));
            }, |mut acc, tokens| {
                let IndexTokens {
                    relative_path,
                    struct_path,
                    kind,
                    ..
                } = tokens;
                let kind_ident = kind.ident();
                let entry_struct_path = ctx.settings.entry_param_for(*kind, ctx.level);
                acc.extend(quote! {
                    map.insert(#relative_path, ::embed_it::Entry::#kind_ident(#entry_struct_path(&#struct_path)));
                });
                acc
            });
        let index = quote! {
            let mut map = ::std::collections::HashMap::with_capacity(#index_len);
            #index
            map
        };
        let value_get = if ctx.settings.support_alt_separator.as_bool() {
            quote! {
                VALUE.get(path.replace("\\", "/").as_str())
            }
        } else {
            quote! {
                VALUE.get(path)
            }
        };

        Ok(quote! {
            fn get(&self, path: &str) -> Option<&'static #entry_path> {
                static VALUE: ::std::sync::LazyLock<::std::collections::HashMap<&'static str, #entry_path>> = ::std::sync::LazyLock::new(|| {
                    #index
                });
                #value_get
            }
        })
    }

    fn definition(&self, _: &GenerationSettings) -> Option<proc_macro2::TokenStream> {
        None
    }

    fn id(&self) -> &'static str {
        "Index"
    }
}
