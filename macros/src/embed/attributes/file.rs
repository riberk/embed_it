use std::fmt::Display;

use darling::FromMeta;
use quote::quote;
use syn::Ident;

use crate::{
    embed::{EntryTokens, GenerateContext},
    embedded_traits::{
        content::ContentTrait, debug::DebugTrait, hashes::ids::*, meta::MetaTrait, path::PathTrait,
        EmbeddedTrait, ResolveEmbeddedTraitError, TraitAttr, EMBEDED_TRAITS,
    },
    main_trait_data::{MainTrait, MainTraitData},
    marker_traits::{child_of::ChildOfMarker, MarkerTrait},
};

use super::{
    derive_default_traits::DeriveDefaultTraits,
    field::{CreateFieldTraitsError, FieldAttr, FieldTraits},
};

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

    #[darling(default, multiple, rename = "field")]
    fields: Vec<FieldAttr>,

    #[darling(default, multiple, rename = "mark")]
    markers: Vec<FileMarkerTrait>,
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

#[derive(Debug, FromMeta, Clone, Copy, PartialEq, Eq)]
pub enum FileMarkerTrait {
    #[darling(rename = "ChildOf")]
    ChildOf,
}

impl From<FileMarkerTrait> for &'static dyn MarkerTrait {
    fn from(value: FileMarkerTrait) -> Self {
        match value {
            FileMarkerTrait::ChildOf => &ChildOfMarker,
        }
    }
}

#[derive(Debug)]
pub struct FileTrait {
    fields: FieldTraits,
    embedded_traits: Vec<&'static dyn EmbeddedTrait>,
    markers: Vec<&'static dyn MarkerTrait>,
    trait_name: Ident,
    field_factory_trait_name: Ident,
}

#[derive(Debug)]
pub enum ParseFileAttrError {
    ResolveEmbeddedTrait(ResolveEmbeddedTraitError),
    CreateFieldTraits(CreateFieldTraitsError),
}

impl From<ResolveEmbeddedTraitError> for ParseFileAttrError {
    fn from(value: ResolveEmbeddedTraitError) -> Self {
        Self::ResolveEmbeddedTrait(value)
    }
}

impl From<CreateFieldTraitsError> for ParseFileAttrError {
    fn from(value: CreateFieldTraitsError) -> Self {
        Self::CreateFieldTraits(value)
    }
}

impl Display for ParseFileAttrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseFileAttrError::ResolveEmbeddedTrait(e) => {
                write!(f, "Unable to resolve embedded trait: {e}")
            }
            ParseFileAttrError::CreateFieldTraits(e) => {
                write!(f, "Unable to create field traits: {e}")
            }
        }
    }
}

impl MainTrait for FileTrait {
    type Trait = FileEmbeddedTrait;

    type Marker = FileMarkerTrait;

    type Error = ParseFileAttrError;

    const DEFAULT_TRAITS: &[&'static dyn EmbeddedTrait] =
        &[&ContentTrait, &DebugTrait, &MetaTrait, &PathTrait];

    const DEFAULT_TRAIT_NAME: &str = "File";

    const DEFAULT_FIELD_FACTORY_TRAIT_NAME: &str = "FileFieldFactory";
}

impl From<MainTraitData> for FileTrait {
    fn from(value: MainTraitData) -> Self {
        let MainTraitData {
            embedded_traits,
            trait_name,
            field_factory_trait_name,
            fields,
            markers,
        } = value;
        Self {
            fields,
            embedded_traits,
            trait_name,
            field_factory_trait_name,
            markers,
        }
    }
}

impl TryFrom<FileAttr> for FileTrait {
    type Error = <Self as MainTrait>::Error;
    fn try_from(value: FileAttr) -> Result<FileTrait, Self::Error> {
        Self::create(
            value.derive_default_traits,
            value.embedded_traits,
            value.markers,
            value.trait_name,
            value.field_factory_trait_name,
            value.fields,
        )
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

    fn fields(&self) -> &FieldTraits {
        &self.fields
    }

    fn markers(&self) -> impl Iterator<Item = &'static dyn MarkerTrait> {
        self.markers.iter().copied()
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
