use darling::FromMeta;
use proc_macro2::Span;
use quote::quote;
use syn::Ident;

use crate::{
    embed::{EntryTokens, GenerateContext},
    embedded_traits::{
        debug::DebugTrait, entries::EntriesTrait, hashes::ids::*, index::IndexTrait,
        meta::MetaTrait, path::PathTrait, EmbeddedTrait, ResolveEmbeddedTraitError, TraitAttr,
        EMBEDED_TRAITS,
    },
};

fn default_dir_traits() -> Vec<DirEmbeddedTrait> {
    Vec::from([
        DirEmbeddedTrait::Debug,
        DirEmbeddedTrait::Entries,
        DirEmbeddedTrait::Index,
        DirEmbeddedTrait::Meta,
        DirEmbeddedTrait::Path,
    ])
}

#[derive(Debug, FromMeta)]
pub struct DirAttr {
    #[darling(default)]
    trait_name: Option<Ident>,

    #[darling(default = default_dir_traits, multiple, rename = "derive")]
    embedded_traits: Vec<DirEmbeddedTrait>,

    #[darling(default)]
    field_factory_trait_name: Option<Ident>,
}

impl Default for DirAttr {
    fn default() -> Self {
        Self {
            trait_name: None,
            embedded_traits: default_dir_traits(),
            field_factory_trait_name: None,
        }
    }
}

#[derive(Debug, FromMeta, Clone, Copy, PartialEq, Eq)]
pub enum DirEmbeddedTrait {
    #[darling(rename = "Path")]
    Path,

    #[darling(rename = "Entries")]
    Entries,

    #[darling(rename = "Index")]
    Index,

    #[darling(rename = "Meta")]
    Meta,

    #[darling(rename = "Debug")]
    Debug,

    #[darling(rename = "Md5")]
    Md5,

    #[darling(rename = "Sha1")]
    Sha1,

    #[darling(rename = "Sha2_224")]
    Sha2_224,

    #[darling(rename = "Sha2_256")]
    Sha2_256,

    #[darling(rename = "Sha2_384")]
    Sha2_384,

    #[darling(rename = "Sha2_512")]
    Sha2_512,

    #[darling(rename = "Sha3_224")]
    Sha3_224,

    #[darling(rename = "Sha3_256")]
    Sha3_256,

    #[darling(rename = "Sha3_384")]
    Sha3_384,

    #[darling(rename = "Sha3_512")]
    Sha3_512,

    #[darling(rename = "Blake3")]
    Blake3,
}

impl DirEmbeddedTrait {
    fn to_embedded_trait(self) -> Result<&'static dyn EmbeddedTrait, ResolveEmbeddedTraitError> {
        match self {
            Self::Path => Ok(&PathTrait),
            Self::Entries => Ok(&EntriesTrait),
            Self::Index => Ok(&IndexTrait),
            Self::Meta => Ok(&MetaTrait),
            Self::Debug => Ok(&DebugTrait),

            Self::Md5 => EMBEDED_TRAITS.get_hash_trait(MD5).map_err(Into::into),
            Self::Sha1 => EMBEDED_TRAITS.get_hash_trait(SHA1).map_err(Into::into),
            Self::Sha2_224 => EMBEDED_TRAITS.get_hash_trait(SHA2_224).map_err(Into::into),
            Self::Sha2_256 => EMBEDED_TRAITS.get_hash_trait(SHA2_256).map_err(Into::into),
            Self::Sha2_384 => EMBEDED_TRAITS.get_hash_trait(SHA2_384).map_err(Into::into),
            Self::Sha2_512 => EMBEDED_TRAITS.get_hash_trait(SHA2_512).map_err(Into::into),
            Self::Sha3_224 => EMBEDED_TRAITS.get_hash_trait(SHA3_224).map_err(Into::into),
            Self::Sha3_256 => EMBEDED_TRAITS.get_hash_trait(SHA3_256).map_err(Into::into),
            Self::Sha3_384 => EMBEDED_TRAITS.get_hash_trait(SHA3_384).map_err(Into::into),
            Self::Sha3_512 => EMBEDED_TRAITS.get_hash_trait(SHA3_512).map_err(Into::into),
            Self::Blake3 => EMBEDED_TRAITS.get_hash_trait(BLAKE3).map_err(Into::into),
        }
    }
}

#[derive(Debug)]
pub struct DirTrait {
    embedded_traits: Vec<&'static dyn EmbeddedTrait>,
    trait_name: Ident,
    field_factory_trait_name: Ident,
}

impl TryFrom<DirAttr> for DirTrait {
    type Error = ResolveEmbeddedTraitError;
    fn try_from(value: DirAttr) -> Result<Self, Self::Error> {
        let res = Self {
            embedded_traits: value
                .embedded_traits
                .into_iter()
                .map(|v| v.to_embedded_trait())
                .collect::<Result<_, _>>()?,
            trait_name: value
                .trait_name
                .unwrap_or_else(|| Ident::new("Dir", Span::call_site())),
            field_factory_trait_name: value
                .field_factory_trait_name
                .unwrap_or_else(|| Ident::new("DirFieldFactory", Span::call_site())),
        };
        Ok(res)
    }
}

impl TraitAttr for DirTrait {
    fn trait_ident(&self) -> &Ident {
        &self.trait_name
    }

    fn field_factory_trait_ident(&self) -> &Ident {
        &self.field_factory_trait_name
    }

    fn embedded_traits(&self) -> impl Iterator<Item = &'static dyn EmbeddedTrait> {
        self.embedded_traits.iter().copied()
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

    use crate::embed::attributes::dir::{default_dir_traits, DirEmbeddedTrait};

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
        assert_eq!(
            result.embedded_traits,
            vec![DirEmbeddedTrait::Path, DirEmbeddedTrait::Entries]
        );
    }

    #[test]
    fn default_fields() {
        let meta: syn::Meta = parse_quote!(dir());

        let result = DirAttr::from_meta(&meta).unwrap();

        assert_eq!(result.trait_name, None);
        assert_eq!(result.field_factory_trait_name, None);
        assert_eq!(result.embedded_traits, default_dir_traits());
    }

    #[test]
    fn default() {
        let result = DirAttr::default();

        assert_eq!(result.trait_name, None);
        assert_eq!(result.field_factory_trait_name, None);
        assert_eq!(result.embedded_traits, default_dir_traits());
    }
}
