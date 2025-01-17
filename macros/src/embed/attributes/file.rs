use darling::FromMeta;
use quote::quote;
use syn::Ident;

use crate::{
    embed::{EntryTokens, GenerateContext},
    embedded_traits::{
        content::ContentTrait, debug::DebugTrait, meta::MetaTrait, path::PathTrait, EmbeddedTrait,
        TraitAttr,
    },
};

fn default_file_traits() -> Vec<FileTrait> {
    Vec::from([
        FileTrait::Content,
        FileTrait::Debug,
        FileTrait::Meta,
        FileTrait::Path,
    ])
}

#[derive(Debug, FromMeta)]
pub struct FileAttr {
    #[darling(default)]
    trait_name: Option<Ident>,

    #[darling(default = default_file_traits, multiple, rename = "derive")]
    traits: Vec<FileTrait>,

    #[darling(default)]
    field_factory_trait_name: Option<Ident>,
}

impl Default for FileAttr {
    fn default() -> Self {
        Self {
            trait_name: None,
            traits: default_file_traits(),
            field_factory_trait_name: None,
        }
    }
}

#[derive(Debug, FromMeta, Clone, Copy, PartialEq, Eq)]
#[darling(rename_all = "PascalCase")]
pub enum FileTrait {
    Path,
    Content,
    Meta,
    Debug,
}

impl FileTrait {
    fn as_embedded_trait(&self) -> &'static dyn EmbeddedTrait {
        match self {
            FileTrait::Path => &PathTrait,
            FileTrait::Content => &ContentTrait,
            FileTrait::Meta => &MetaTrait,
            FileTrait::Debug => &DebugTrait,
        }
    }
}

impl TraitAttr for FileAttr {
    const DEFAULT_TRAIT_NAME: &str = "File";
    const DEFAULT_FIELD_FACTORY_TRAIT_NAME: &str = "FileFieldFactory";

    fn trait_name(&self) -> Option<&Ident> {
        self.trait_name.as_ref()
    }

    fn field_factory_trait_name(&self) -> Option<&Ident> {
        self.field_factory_trait_name.as_ref()
    }

    fn traits(&self) -> impl Iterator<Item = &'static dyn EmbeddedTrait> {
        self.traits.iter().map(|v| v.as_embedded_trait())
    }

    fn struct_impl(&self, _: &GenerateContext<'_>, _: &[EntryTokens]) -> proc_macro2::TokenStream {
        quote! {}
    }
}
