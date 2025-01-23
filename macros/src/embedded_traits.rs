pub mod content;
pub mod debug;
pub mod direct_child_count;
pub mod entries;
pub mod hashes;
pub mod index;
pub mod meta;
pub mod path;
pub mod recursive_child_count;

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    error::Error,
    fmt::{Debug, Display},
    sync::LazyLock,
};

use hashes::ids;
use quote::quote;
use syn::{parse_quote, punctuated::Punctuated, Ident, Token, TraitBound, TypeParamBound};

use crate::{
    embed::{
        attributes::{derive_default_traits::DeriveDefaultTraits, field::FieldTraits},
        bool_like_enum::BoolLikeEnum,
        EntryTokens, GenerateContext, IndexTokens,
    },
    fs::EntryKind,
    marker_traits::MarkerTrait,
};

pub struct AllEmbededTraits(HashMap<&'static str, &'static dyn EmbeddedTrait>);

impl Default for AllEmbededTraits {
    fn default() -> Self {
        let mut map = Self(HashMap::new());
        map.add(&content::ContentTrait);
        map.add(&debug::DebugTrait);
        map.add(&entries::EntriesTrait);
        map.add(&index::IndexTrait);
        map.add(&meta::MetaTrait);
        map.add(&path::PathTrait);
        map.add(&direct_child_count::DirectChildCountTrait);
        map.add(&recursive_child_count::RecursiveChildCountTrait);

        #[cfg(feature = "md5")]
        map.add(hashes::md5::MD5);

        #[cfg(feature = "sha1")]
        map.add(hashes::sha1::SHA1);

        #[cfg(feature = "sha2")]
        {
            map.add(hashes::sha2::SHA2_224);
            map.add(hashes::sha2::SHA2_256);
            map.add(hashes::sha2::SHA2_384);
            map.add(hashes::sha2::SHA2_512);
        }

        #[cfg(feature = "sha3")]
        {
            map.add(hashes::sha3::SHA3_224);
            map.add(hashes::sha3::SHA3_256);
            map.add(hashes::sha3::SHA3_384);
            map.add(hashes::sha3::SHA3_512);
        }

        #[cfg(feature = "blake3")]
        map.add(hashes::blake3::BLAKE3);

        map
    }
}

impl AllEmbededTraits {
    fn add<T: EmbeddedTrait>(&mut self, t: &'static T) {
        let res = self.0.insert(t.id(), t);
        if res.is_some() {
            panic!("Duplicate trait id '{}'", t.id(),);
        }
    }

    pub fn get_hash_trait(
        &self,
        id: &'static ids::AlgId,
    ) -> Result<&'static dyn EmbeddedTrait, FeatureDisabled> {
        self.0
            .get(id.id)
            .ok_or(FeatureDisabled {
                requested: id.id,
                feature: id.feature,
            })
            .copied()
    }

    pub fn get(&self, id: &str) -> Option<&'static dyn EmbeddedTrait> {
        self.0.get(id).copied()
    }
}

pub static EMBEDED_TRAITS: LazyLock<AllEmbededTraits> = LazyLock::new(AllEmbededTraits::default);

#[derive(Debug)]
pub enum ResolveEmbeddedTraitError {
    FeatureDisabled(FeatureDisabled),
}

impl Display for ResolveEmbeddedTraitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveEmbeddedTraitError::FeatureDisabled(e) => write!(f, "{e}"),
        }
    }
}

impl From<FeatureDisabled> for ResolveEmbeddedTraitError {
    fn from(value: FeatureDisabled) -> Self {
        Self::FeatureDisabled(value)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct FeatureDisabled {
    requested: &'static str,
    feature: &'static str,
}

impl Display for FeatureDisabled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { requested, feature } = self;
        write!(
            f,
            "Feature '{feature}' must be enabled to use '{requested}'"
        )
    }
}

pub trait EmbeddedTrait: Send + Sync + Debug {
    fn id(&self) -> &'static str;

    fn path(&self, nesting: usize) -> syn::Path;

    /// Definition of the trait. If it is external trait (like Debug) it returns None
    fn definition(&self, entry_path: &syn::Ident) -> Option<proc_macro2::TokenStream>;

    fn bound(&self) -> TraitBound {
        let path = self.path(0);
        parse_quote!(#path)
    }

    fn impl_body(
        &self,
        ctx: &mut GenerateContext<'_>,
        entries: &[EntryTokens],
        index: &[IndexTokens],
    ) -> Result<proc_macro2::TokenStream, MakeEmbeddedTraitImplementationError>;

    fn implementation(
        &self,
        ctx: &mut GenerateContext<'_>,
        entries: &[EntryTokens],
        index: &[IndexTokens],
    ) -> Result<proc_macro2::TokenStream, MakeEmbeddedTraitImplementationError> {
        let trait_path = self.path(ctx.level);
        let body = self.impl_body(ctx, entries, index)?;
        let struct_ident = &ctx.struct_ident;

        Ok(quote! {
            #[automatically_derived]
            impl #trait_path for #struct_ident {
                #body
            }
        })
    }

    fn entry_impl_body(&self) -> proc_macro2::TokenStream;
}

#[derive(Debug)]
pub enum MakeEmbeddedTraitImplementationError {
    Custom(
        #[allow(dead_code)] Cow<'static, str>,
        #[allow(dead_code)] Option<Box<dyn Error>>,
    ),
    UnsupportedEntry {
        #[allow(dead_code)]
        entry: EntryKind,

        #[allow(dead_code)]
        trait_id: &'static str,
    },
}

impl MakeEmbeddedTraitImplementationError {
    pub fn with_error(message: impl Into<Cow<'static, str>>, e: impl Error + 'static) -> Self {
        Self::Custom(message.into(), Some(Box::new(e)))
    }
}

pub trait TraitAttr {
    fn trait_ident(&self) -> &Ident;

    fn field_factory_trait_ident(&self) -> &Ident;

    /// Which traits must be implemented for any of implementors of that trait
    fn embedded_traits(&self) -> impl Iterator<Item = &'static dyn EmbeddedTrait>;

    fn fields(&self) -> &FieldTraits;

    fn markers(&self) -> impl Iterator<Item = &'static dyn MarkerTrait>;

    fn struct_impl(
        &self,
        ctx: &GenerateContext<'_>,
        entries: &[EntryTokens],
    ) -> proc_macro2::TokenStream;

    /// That trait implements debug
    fn is_trait_implemented(&self, expected: &'static impl EmbeddedTrait) -> bool {
        let expected = expected.id();
        self.embedded_traits().any(|t| t.id() == expected)
    }

    fn definition(&self) -> proc_macro2::TokenStream {
        let trait_ident = self.trait_ident();

        let mut bounds = Punctuated::<TypeParamBound, Token![+]>::new();
        bounds.push(parse_quote!(Send));
        bounds.push(parse_quote!(Sync));

        for t in self.embedded_traits() {
            bounds.push(TypeParamBound::Trait(t.bound()));
        }

        quote! {
            pub trait #trait_ident : #bounds {}
        }
    }

    /// Implements this trait (and its bounds) for an entry
    /// # Arguments
    ///
    /// * `self`
    /// * `ctx` - The context of generated entry
    /// * `entries` - Direct children entries
    /// * `index` - Recursive children including the direct
    fn implementation_stream(
        &self,
        ctx: &mut GenerateContext<'_>,
        entries: &[EntryTokens],
        index: &[IndexTokens],
    ) -> Result<proc_macro2::TokenStream, MakeEmbeddedTraitImplementationError> {
        let trait_ident = self.trait_ident();
        let mut impl_stream = quote! {};

        for t in self.embedded_traits() {
            impl_stream.extend(t.implementation(ctx, entries, index)?);
        }

        for m in self.markers() {
            impl_stream.extend(m.implementation(ctx, entries, index));
        }

        let trait_path = ctx.make_level_path(trait_ident.to_owned());
        let struct_impl = self.struct_impl(ctx, entries);
        let struct_ident = &ctx.struct_ident;
        impl_stream.extend(quote! {
            #struct_impl

            #[automatically_derived]
            impl #trait_path for #struct_ident {}
        });
        Ok(impl_stream)
    }
}

#[derive(Debug, Clone)]
pub struct EnabledTraits(Vec<&'static dyn EmbeddedTrait>);

impl EnabledTraits {
    pub fn create<T: TryInto<&'static dyn EmbeddedTrait>>(
        derive_default: DeriveDefaultTraits,
        defined_traits: Vec<T>,
        defautl_traits: &[&'static dyn EmbeddedTrait],
    ) -> Result<Self, T::Error> {
        let mut enabled_traits = HashSet::new();
        let mut embedded_traits = Vec::new();

        for embedded_trait in defined_traits {
            let embedded_trait = embedded_trait.try_into()?;
            enabled_traits.insert(embedded_trait.id());
            embedded_traits.push(embedded_trait);
        }

        let default_traits = derive_default
            .as_bool()
            .then_some(defautl_traits)
            .unwrap_or_default();
        for &default_trait in default_traits {
            if !enabled_traits.contains(default_trait.id()) {
                enabled_traits.insert(default_trait.id());
                embedded_traits.push(default_trait);
            }
        }

        Ok(EnabledTraits(embedded_traits))
    }
}

impl From<EnabledTraits> for Vec<&'static dyn EmbeddedTrait> {
    fn from(value: EnabledTraits) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use crate::embed::attributes::derive_default_traits::DeriveDefaultTraits;

    use super::{
        entries::EntriesTrait, index::IndexTrait, path::PathTrait, AllEmbededTraits, EmbeddedTrait,
        EnabledTraits,
    };

    #[test]
    fn create_with_duplicates() {
        let defined_traits: Vec<&'static dyn EmbeddedTrait> = vec![&PathTrait, &IndexTrait];
        let res = EnabledTraits::create(
            DeriveDefaultTraits::Yes,
            defined_traits,
            &[&PathTrait, &EntriesTrait],
        )
        .unwrap();
        let ids = res.0.into_iter().map(|v| v.id()).collect::<Vec<_>>();
        assert_eq!(&ids, &["Path", "Index", "Entries"]);
    }

    #[test]
    fn create_with_error() {
        const ERROR: &str = "fseljabskrbgkhsdbgsd";
        pub struct A;
        impl TryFrom<A> for &'static dyn EmbeddedTrait {
            type Error = &'static str;

            fn try_from(_value: A) -> Result<Self, Self::Error> {
                Err(ERROR)
            }
        }

        let err = EnabledTraits::create(DeriveDefaultTraits::No, vec![A], &[]).unwrap_err();

        assert_eq!(err, ERROR);
    }

    #[test]
    #[should_panic(expected = r#"Duplicate trait id 'Path'"#)]
    fn all_embedded_traits_add_duplicate() {
        let mut traits = AllEmbededTraits::default();
        traits.add(&PathTrait);
        traits.add(&PathTrait);
    }

    #[test]
    #[cfg(not(feature = "md5"))]
    fn all_embedded_traits_get_md5_feature_error() {
        use super::FeatureDisabled;
        use crate::embedded_traits::hashes::ids::MD5;

        let err = AllEmbededTraits::default().get_hash_trait(MD5).unwrap_err();
        assert_eq!(
            err,
            FeatureDisabled {
                requested: "Hash(md5)",
                feature: "md5"
            }
        );
    }
}
