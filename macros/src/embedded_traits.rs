pub mod content;
pub mod debug;
pub mod direct_child_count;
pub mod enabled_trait;
pub mod entries;
pub mod feature_disabled;
pub mod hashes;
pub mod index;
pub mod main_trait;
pub mod meta;
pub mod path;
pub mod recursive_child_count;

use std::{
    borrow::Cow,
    collections::HashSet,
    error::Error,
    fmt::{Debug, Display},
};

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

pub trait EmbeddedTrait: Debug + 'static {
    fn id(&self) -> &str;

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

#[cfg(test)]
mod tests {
    use crate::{embed::attributes::derive_default_traits::DeriveDefaultTraits, embedded_traits::enabled_trait::EnabledTraits};

    use super::{
        entries::EntriesTrait, index::IndexTrait, path::PathTrait, EmbeddedTrait,
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
}
