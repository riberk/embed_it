use darling::FromMeta;
use quote::quote;
use syn::Ident;

use crate::{
    embed::{EntryTokens, GenerateContext},
    embedded_traits::{
        debug::DebugTrait, entries::EntriesTrait, index::IndexTrait, meta::MetaTrait,
        path::PathTrait, EmbeddedTrait, TraitAttr,
    },
};

fn default_dir_traits() -> Vec<DirTrait> {
    Vec::from([
        DirTrait::Debug,
        DirTrait::Entries,
        DirTrait::Index,
        DirTrait::Meta,
        DirTrait::Path,
    ])
}

#[derive(Debug, FromMeta)]
pub struct DirAttr {
    #[darling(default)]
    trait_name: Option<Ident>,

    #[darling(default = default_dir_traits, multiple, rename = "derive")]
    traits: Vec<DirTrait>,

    #[darling(default)]
    field_factory_trait_name: Option<Ident>,
}

impl Default for DirAttr {
    fn default() -> Self {
        Self {
            trait_name: None,
            traits: default_dir_traits(),
            field_factory_trait_name: None,
        }
    }
}

#[derive(Debug, FromMeta, Clone, Copy, PartialEq, Eq)]
#[darling(rename_all = "PascalCase")]
pub enum DirTrait {
    Path,
    Entries,
    Index,
    Meta,
    Debug,
}

impl DirTrait {
    fn as_embedded_trait(&self) -> &'static dyn EmbeddedTrait {
        match self {
            DirTrait::Path => &PathTrait,
            DirTrait::Entries => &EntriesTrait,
            DirTrait::Index => &IndexTrait,
            DirTrait::Meta => &MetaTrait,
            DirTrait::Debug => &DebugTrait,
        }
    }
}

impl TraitAttr for DirAttr {
    const DEFAULT_TRAIT_NAME: &str = "Dir";
    const DEFAULT_FIELD_FACTORY_TRAIT_NAME: &str = "DirFieldFactory";

    fn trait_name(&self) -> Option<&Ident> {
        self.trait_name.as_ref()
    }

    fn field_factory_trait_name(&self) -> Option<&Ident> {
        self.field_factory_trait_name.as_ref()
    }

    fn traits(&self) -> impl Iterator<Item = &'static dyn EmbeddedTrait> {
        self.traits.iter().map(|v| v.as_embedded_trait())
    }

    fn struct_impl(
        &self,
        ctx: &GenerateContext<'_>,
        entries: &[EntryTokens],
    ) -> proc_macro2::TokenStream {
        let struct_ident = &ctx.struct_ident;
        let methods = entries.iter().fold(quote! {}, |mut acc, entry| {
            let EntryTokens {
                struct_path,
                field_ident,
                ..
            } = entry;
            acc.extend(quote! {
                pub fn #field_ident(&self) -> &'static #struct_path {
                    &#struct_path
                }
            });
            acc
        });

        quote! {
            impl #struct_ident {
                #methods
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use darling::FromMeta;
    use proc_macro2::Span;
    use syn::{parse_quote, Ident};

    use crate::embed::attributes::dir::{default_dir_traits, DirTrait};

    use super::DirAttr;

    #[test]
    fn parse_all_fields() {
        let meta: syn::Meta = parse_quote!(dir(
            trait_name = TraitName,
            field_factory_trait_name = FieldFactory,
            derive(Path),
            derive(Entries)
        ));

        let result = DirAttr::from_meta(&meta).unwrap();

        assert_eq!(
            result.trait_name,
            Some(Ident::new("TraitName", Span::call_site()))
        );
        assert_eq!(
            result.field_factory_trait_name,
            Some(Ident::new("FieldFactory", Span::call_site()))
        );
        assert_eq!(result.traits, vec![DirTrait::Path, DirTrait::Entries]);
    }

    #[test]
    fn default_fields() {
        let meta: syn::Meta = parse_quote!(dir());

        let result = DirAttr::from_meta(&meta).unwrap();

        assert_eq!(result.trait_name, None);
        assert_eq!(result.field_factory_trait_name, None);
        assert_eq!(result.traits, default_dir_traits());
    }

    #[test]
    fn default() {
        let result = DirAttr::default();

        assert_eq!(result.trait_name, None);
        assert_eq!(result.field_factory_trait_name, None);
        assert_eq!(result.traits, default_dir_traits());
    }
}
