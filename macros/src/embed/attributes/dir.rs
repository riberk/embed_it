use darling::FromMeta;
use quote::quote;
use syn::Ident;

use crate::{
    embed::{EntryTokens, GenerateContext},
    embedded_traits::{
        EMBEDED_TRAITS, EmbeddedTrait, ResolveEmbeddedTraitError, TraitAttr, debug::DebugTrait,
        direct_child_count::DirectChildCountTrait, entries::EntriesTrait, hashes::ids::*,
        index::IndexTrait, meta::MetaTrait, path::PathTrait,
        recursive_child_count::RecursiveChildCountTrait,
    },
    main_trait_data::{MainTrait, MainTraitData},
    marker_traits::{MarkerTrait, child_of::ChildOfMarker},
};

use super::{
    derive_default_traits::DeriveDefaultTraits,
    field::{CreateFieldTraitsError, FieldAttr, FieldTraits},
    path_match::{PathMatcher, PathMatcherAttr},
};

#[derive(Debug, FromMeta, Default)]
pub struct DirAttr {
    #[darling(default, rename = "derive_default_traits")]
    derive_default_traits: DeriveDefaultTraits,

    #[darling(default)]
    trait_name: Option<Ident>,

    #[darling(multiple, rename = "derive")]
    embedded_traits: Vec<DirEmbeddedTrait>,

    #[darling(default)]
    field_factory_trait_name: Option<Ident>,

    #[darling(default, multiple, rename = "field")]
    fields: Vec<FieldAttr>,

    #[darling(default, multiple, rename = "mark")]
    markers: Vec<DirMarkerTrait>,

    #[darling(default, flatten)]
    matcher: PathMatcherAttr,
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

    #[darling(rename = "DirectChildCount")]
    DirectChildCount,

    #[darling(rename = "RecursiveChildCount")]
    RecursiveChildCount,

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
            Self::DirectChildCount => Ok(&DirectChildCountTrait),
            Self::RecursiveChildCount => Ok(&RecursiveChildCountTrait),

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

impl TryFrom<DirEmbeddedTrait> for &'static dyn EmbeddedTrait {
    type Error = ResolveEmbeddedTraitError;

    fn try_from(value: DirEmbeddedTrait) -> Result<Self, Self::Error> {
        value.to_embedded_trait()
    }
}

#[derive(Debug, FromMeta, Clone, Copy, PartialEq, Eq)]
pub enum DirMarkerTrait {
    #[darling(rename = "ChildOf")]
    ChildOf,
}

impl From<DirMarkerTrait> for &'static dyn MarkerTrait {
    fn from(value: DirMarkerTrait) -> Self {
        match value {
            DirMarkerTrait::ChildOf => &ChildOfMarker,
        }
    }
}

#[derive(Debug)]
pub struct DirTrait {
    embedded_traits: Vec<&'static dyn EmbeddedTrait>,
    trait_name: Ident,
    field_factory_trait_name: Ident,
    fields: FieldTraits,
    markers: Vec<&'static dyn MarkerTrait>,
    matcher: PathMatcher,
}

impl MainTrait for DirTrait {
    type Trait = DirEmbeddedTrait;
    type Marker = DirMarkerTrait;
    type Error = ParseDirAttrError;

    const DEFAULT_TRAITS: &[&'static dyn EmbeddedTrait] = &[
        &DebugTrait,
        &EntriesTrait,
        &IndexTrait,
        &MetaTrait,
        &PathTrait,
        &DirectChildCountTrait,
        &RecursiveChildCountTrait,
    ];

    const DEFAULT_TRAIT_NAME: &str = "Dir";
    const DEFAULT_FIELD_FACTORY_TRAIT_NAME: &str = "DirFieldFactory";
}

impl From<MainTraitData> for DirTrait {
    fn from(value: MainTraitData) -> Self {
        let MainTraitData {
            embedded_traits,
            trait_name,
            field_factory_trait_name,
            fields,
            markers,
            matcher,
        } = value;
        Self {
            embedded_traits,
            trait_name,
            field_factory_trait_name,
            fields,
            markers,
            matcher,
        }
    }
}

#[derive(Debug, derive_more::Display, derive_more::From)]
pub enum ParseDirAttrError {
    #[display("unable to resolve embedded trait: {_0}")]
    ResolveEmbeddedTrait(ResolveEmbeddedTraitError),

    #[display("unable to create field traits: {_0}")]
    CreateFieldTraits(CreateFieldTraitsError),
}

impl TryFrom<DirAttr> for DirTrait {
    type Error = <Self as MainTrait>::Error;
    fn try_from(value: DirAttr) -> Result<Self, Self::Error> {
        Self::create(
            value.derive_default_traits,
            value.embedded_traits,
            value.markers,
            value.trait_name,
            value.field_factory_trait_name,
            value.fields,
            value.matcher,
        )
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

    fn fields(&self) -> &FieldTraits {
        &self.fields
    }

    fn markers(&self) -> impl Iterator<Item = &'static dyn MarkerTrait> {
        self.markers.iter().copied()
    }

    fn entry_trait_ident<'a>(&self, entry: &'a super::entry::EntryStruct) -> &'a Ident {
        entry.dir_trait_ident()
    }

    fn entry_struct_ident<'a>(&self, entry: &'a super::entry::EntryStruct) -> &'a Ident {
        entry.dir_struct_ident()
    }

    fn should_be_included(&self, path: &crate::fs::EntryPath) -> bool {
        self.matcher.should_be_included(path)
    }

    fn struct_impl(
        &self,
        ctx: &GenerateContext<'_>,
        entries: &[EntryTokens],
    ) -> proc_macro2::TokenStream {
        let struct_ident = &ctx.entry_struct_ident();
        let methods = entries.iter().fold(quote! {}, |mut acc, entry| {
            let EntryTokens {
                struct_path, field, ..
            } = entry;
            acc.extend(quote! {
                pub fn #field(&self) -> &'static #struct_path {
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
    use syn::{Ident, parse_quote};

    use crate::embed::attributes::{
        derive_default_traits::DeriveDefaultTraits, dir::DirEmbeddedTrait,
    };

    use super::DirAttr;

    #[test]
    fn parse_all_fields() {
        let meta: syn::Meta = parse_quote!(dir(
            derive_default_traits = false,
            trait_name = TraitName,
            field_factory_trait_name = FieldFactory,
            derive(Path),
            derive(Entries)
        ));

        let result = DirAttr::from_meta(&meta).unwrap();

        assert_eq!(result.derive_default_traits, DeriveDefaultTraits::No,);

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

        assert_eq!(result.derive_default_traits, DeriveDefaultTraits::Yes);
        assert_eq!(result.trait_name, None);
        assert_eq!(result.field_factory_trait_name, None);
        assert_eq!(result.embedded_traits, Vec::default());
    }

    #[test]
    fn default() {
        let result = DirAttr::default();

        assert_eq!(result.derive_default_traits, DeriveDefaultTraits::Yes);
        assert_eq!(result.trait_name, None);
        assert_eq!(result.field_factory_trait_name, None);
        assert_eq!(result.embedded_traits, Vec::default());
    }
}
