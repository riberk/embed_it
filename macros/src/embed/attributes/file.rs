use darling::FromMeta;
use proc_macro2::Span;
use quote::quote;
use syn::Ident;

use crate::{
    embed::{EntryTokens, GenerateContext},
    embedded_traits::{
        content::ContentTrait, debug::DebugTrait, hashes::ids::*, meta::MetaTrait, path::PathTrait,
        EmbeddedTrait, EnabledTraits, ResolveEmbeddedTraitError, TraitAttr, EMBEDED_TRAITS,
    },
};

use super::derive_default_traits::DeriveDefaultTraits;

#[derive(Debug, FromMeta, Default)]
pub struct FileAttr {
    #[darling(default, rename = "derive_default_traits")]
    derive_default_traits: DeriveDefaultTraits,

    #[darling(default)]
    trait_name: Option<Ident>,

    #[darling(multiple, rename = "derive")]
    embedded_traits: Vec<FileEmbeddedTrait>,

    #[darling(default)]
    field_factory_trait_name: Option<Ident>,
}

#[derive(Debug, FromMeta, Clone, Copy, PartialEq, Eq)]
pub enum FileEmbeddedTrait {
    #[darling(rename = "Path")]
    Path,

    #[darling(rename = "Content")]
    Content,

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

impl FileEmbeddedTrait {
    fn to_embedded_trait(self) -> Result<&'static dyn EmbeddedTrait, ResolveEmbeddedTraitError> {
        match self {
            Self::Path => Ok(&PathTrait),
            Self::Content => Ok(&ContentTrait),
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

impl TryFrom<FileEmbeddedTrait> for &'static dyn EmbeddedTrait {
    type Error = ResolveEmbeddedTraitError;

    fn try_from(value: FileEmbeddedTrait) -> Result<Self, Self::Error> {
        value.to_embedded_trait()
    }
}

const DEFAULT_TRAITS: &[&'static dyn EmbeddedTrait] =
    &[&ContentTrait, &DebugTrait, &MetaTrait, &PathTrait];

#[derive(Debug)]
pub struct FileTrait {
    embedded_traits: Vec<&'static dyn EmbeddedTrait>,
    trait_name: Ident,
    field_factory_trait_name: Ident,
}

impl TryFrom<FileAttr> for FileTrait {
    type Error = ResolveEmbeddedTraitError;
    fn try_from(value: FileAttr) -> Result<FileTrait, Self::Error> {
        let enabled_traits = EnabledTraits::create(
            value.derive_default_traits,
            value.embedded_traits,
            DEFAULT_TRAITS,
        )?;

        let res = Self {
            embedded_traits: enabled_traits.into(),
            trait_name: value
                .trait_name
                .unwrap_or_else(|| Ident::new("File", Span::call_site())),
            field_factory_trait_name: value
                .field_factory_trait_name
                .unwrap_or_else(|| Ident::new("FileFieldFactory", Span::call_site())),
        };
        Ok(res)
    }
}

impl TraitAttr for FileTrait {
    fn trait_ident(&self) -> &Ident {
        &self.trait_name
    }

    fn field_factory_trait_ident(&self) -> &Ident {
        &self.field_factory_trait_name
    }

    fn embedded_traits(&self) -> impl Iterator<Item = &'static dyn EmbeddedTrait> {
        self.embedded_traits.iter().copied()
    }

    fn struct_impl(&self, _: &GenerateContext<'_>, _: &[EntryTokens]) -> proc_macro2::TokenStream {
        quote! {}
    }
}

#[cfg(test)]
mod tests {
    use darling::FromMeta;
    use proc_macro2::Span;
    use syn::{parse_quote, Ident};

    use crate::embed::attributes::{
        derive_default_traits::DeriveDefaultTraits, file::FileEmbeddedTrait,
    };

    use super::FileAttr;

    #[test]
    fn parse_all_fields() {
        let meta: syn::Meta = parse_quote!(dir(
            derive_default_traits,
            trait_name = TraitName,
            field_factory_trait_name = FieldFactory,
            derive(Path),
            derive(Content)
        ));

        let result = FileAttr::from_meta(&meta).unwrap();

        assert_eq!(result.derive_default_traits, DeriveDefaultTraits::Yes);
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
            vec![FileEmbeddedTrait::Path, FileEmbeddedTrait::Content]
        );
    }

    #[test]
    fn default_fields() {
        let meta: syn::Meta = parse_quote!(dir());

        let result = FileAttr::from_meta(&meta).unwrap();

        assert_eq!(result.derive_default_traits, DeriveDefaultTraits::Yes);
        assert_eq!(result.trait_name, None);
        assert_eq!(result.field_factory_trait_name, None);
        assert_eq!(result.embedded_traits, Vec::default());
    }

    #[test]
    fn default() {
        let result = FileAttr::default();

        assert_eq!(result.derive_default_traits, DeriveDefaultTraits::Yes);
        assert_eq!(result.trait_name, None);
        assert_eq!(result.field_factory_trait_name, None);
        assert_eq!(result.embedded_traits, Vec::default());
    }
}
