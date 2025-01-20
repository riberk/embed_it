use proc_macro2::Span;
use syn::Ident;

use crate::{
    embed::attributes::{
        derive_default_traits::DeriveDefaultTraits,
        field::{CreateFieldTraitsError, FieldAttr, FieldTraits},
    },
    embedded_traits::{EmbeddedTrait, EnabledTraits},
};

pub struct MainTraitData {
    pub embedded_traits: Vec<&'static dyn EmbeddedTrait>,
    pub trait_name: Ident,
    pub field_factory_trait_name: Ident,
    pub fields: FieldTraits,
}

pub trait MainTrait: Sized + 'static + From<MainTraitData> {
    type Trait: TryInto<&'static dyn EmbeddedTrait>;
    type Error: From<<Self::Trait as TryInto<&'static dyn EmbeddedTrait>>::Error>
        + From<CreateFieldTraitsError>;
    const DEFAULT_TRAITS: &[&'static dyn EmbeddedTrait];
    const DEFAULT_TRAIT_NAME: &str;
    const DEFAULT_FIELD_FACTORY_TRAIT_NAME: &str;

    fn create(
        derive_default_traits: DeriveDefaultTraits,
        embedded_traits: Vec<Self::Trait>,
        trait_name: Option<Ident>,
        field_factory_trait_name: Option<Ident>,
        fields: Vec<FieldAttr>,
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
        };
        Ok(res.into())
    }
}
