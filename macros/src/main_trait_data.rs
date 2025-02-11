use proc_macro2::Span;
use syn::Ident;

use crate::{
    embed::attributes::{
        derive_default_traits::DeriveDefaultTraits,
        field::{CreateFieldTraitsError, FieldAttr, FieldTraits},
        path_match::{PathMatcher, PathMatcherAttr},
    },
    embedded_traits::{EmbeddedTrait, EnabledTraits},
    marker_traits::MarkerTrait,
};

pub struct MainTraitData {
    pub embedded_traits: Vec<&'static dyn EmbeddedTrait>,
    pub trait_name: Ident,
    pub field_factory_trait_name: Ident,
    pub fields: FieldTraits,
    pub markers: Vec<&'static dyn MarkerTrait>,
    pub matcher: PathMatcher,
}

pub trait MainTrait: Sized + 'static + From<MainTraitData> {
    type Trait: TryInto<&'static dyn EmbeddedTrait>;
    type Marker: Into<&'static dyn MarkerTrait>;
    type Error: From<<Self::Trait as TryInto<&'static dyn EmbeddedTrait>>::Error>
        + From<CreateFieldTraitsError>;
    const DEFAULT_TRAITS: &[&'static dyn EmbeddedTrait];
    const DEFAULT_TRAIT_NAME: &str;
    const DEFAULT_FIELD_FACTORY_TRAIT_NAME: &str;

    fn create(
        derive_default_traits: DeriveDefaultTraits,
        embedded_traits: Vec<Self::Trait>,
        markers: Vec<Self::Marker>,
        trait_name: Option<Ident>,
        field_factory_trait_name: Option<Ident>,
        fields: Vec<FieldAttr>,
        matcher: PathMatcherAttr,
    ) -> Result<Self, Self::Error> {
        let enabled_traits =
            EnabledTraits::create(derive_default_traits, embedded_traits, Self::DEFAULT_TRAITS)?;

        let trait_name =
            trait_name.unwrap_or_else(|| Ident::new(Self::DEFAULT_TRAIT_NAME, Span::call_site()));
        let field_factory_trait_name = field_factory_trait_name.unwrap_or_else(|| {
            Ident::new(Self::DEFAULT_FIELD_FACTORY_TRAIT_NAME, Span::call_site())
        });

        let fields = FieldTraits::create(fields)?;
        let res = MainTraitData {
            fields,
            embedded_traits: enabled_traits.into(),
            trait_name,
            field_factory_trait_name,
            markers: markers.into_iter().map(Into::into).collect(),
            matcher: matcher.into(),
        };
        Ok(res.into())
    }
}
