use std::{collections::HashSet, ops::Deref};

use crate::embed::attributes::derive_default_traits::DeriveDefaultTraits;

use super::EmbeddedTrait;

#[derive(Debug)]
pub struct EnabledTrait(Box<dyn EmbeddedTrait>);

impl Deref for EnabledTrait {
    type Target = dyn EmbeddedTrait;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl EnabledTrait {
    pub fn new<T: EmbeddedTrait>(t: T) -> Self {
        Self(Box::new(t))
    }
}

#[derive(Debug)]
pub struct EnabledTraits(Vec<EnabledTrait>);

impl EnabledTraits {
    pub fn create<T: TryInto<EnabledTrait>>(
        derive_default: DeriveDefaultTraits,
        defined_traits: Vec<T>,
        defautl_traits: Vec<EnabledTrait>,
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
        for default_trait in default_traits {
            if !enabled_traits.contains(default_trait.id()) {
                enabled_traits.insert(default_trait.id());
                embedded_traits.push(default_trait);
            }
        }

        Ok(EnabledTraits(embedded_traits))
    }
}

impl From<EnabledTraits> for Vec<EnabledTrait> {
    fn from(value: EnabledTraits) -> Self {
        value.0
    }
}

