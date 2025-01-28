use std::{collections::HashMap, fmt::Display};

use convert_case::{Case, Casing};
use darling::FromMeta;
use proc_macro2::Span;
use quote::quote;
use syn::Ident;

use crate::{
    embed::{fix_path, pattern::EntryPattern, regex::EntryRegex, GenerateContext}, embedded_traits::main_trait::MainTrait, fs::EntryPath
};

#[derive(Debug, FromMeta)]
pub struct FieldAttr {
    regex: Option<EntryRegex>,

    pattern: Option<EntryPattern>,

    factory: syn::Path,

    name: syn::Ident,

    trait_name: Option<syn::Ident>,
}

#[derive(Debug)]
pub struct FieldTrait {
    field_ident: syn::Ident,
    trait_ident: syn::Ident,
    factory: syn::Path,
    regex: Option<EntryRegex>,
    pattern: Option<EntryPattern>,
}

impl FieldTrait {
    pub fn create(field_attr: FieldAttr) -> Self {
        let trait_ident = field_attr.trait_name.unwrap_or_else(|| {
            let name = format!("{}Field", field_attr.name.to_string().to_case(Case::Pascal));
            Ident::new_raw(&name, Span::call_site())
        });

        Self {
            field_ident: field_attr.name,
            trait_ident,
            regex: field_attr.regex,
            pattern: field_attr.pattern,
            factory: field_attr.factory,
        }
    }

    pub fn is_match(&self, path: &EntryPath) -> bool {
        self.regex
            .as_ref()
            .map(|v| v.is_match(path))
            .unwrap_or(true)
            && self
                .pattern
                .as_ref()
                .map(|v| v.is_match(path))
                .unwrap_or(true)
    }

    pub fn definition(&self, generate_for: &impl MainTrait) -> proc_macro2::TokenStream {
        let FieldTrait {
            field_ident,
            trait_ident,
            factory,
            ..
        } = self;
        let bound_ident = generate_for.trait_ident();
        let factory_trait = generate_for.field_factory_trait_ident();
        quote! {
            pub trait #trait_ident: #bound_ident {
                fn #field_ident(&self) -> &'static <#factory as #factory_trait>::Field;
            }
        }
    }

    pub fn implementation(&self, ctx: &GenerateContext<'_>) -> proc_macro2::TokenStream {
        let FieldTrait {
            field_ident,
            trait_ident,
            factory,
            ..
        } = self;

        let struct_ident = &ctx.struct_ident;
        let factory = fix_path(factory, ctx.level);
        let trait_path = ctx.make_level_path(trait_ident.clone());
        let factory_trait = ctx
            .entry_trait()
            .map(
                |v| v.field_factory_trait_ident(),
                |v| v.field_factory_trait_ident(),
            )
            .value();
        let factory_trait_path = ctx.make_level_path(factory_trait.clone());

        quote! {
            #[automatically_derived]
            impl #trait_path for #struct_ident {
                fn #field_ident(&self) -> &'static <#factory as #factory_trait_path>::Field {
                    static VALUE: ::std::sync::OnceLock<<#factory as #factory_trait_path>::Field> = ::std::sync::OnceLock::new();

                    VALUE.get_or_init(|| {
                        <#factory as #factory_trait_path>::create(self)
                    })
                }
            }
        }
    }

    pub fn trait_ident(&self) -> &syn::Ident {
        &self.trait_ident
    }

    pub fn field_ident(&self) -> &syn::Ident {
        &self.field_ident
    }
}

#[derive(Debug)]
pub struct FieldTraits {
    traits: Vec<FieldTrait>,
    by_trait_name: HashMap<Ident, usize>,
}

#[derive(Debug)]
pub enum CreateFieldTraitsError {
    DuplicateTraitName(DuplicateTraitName),
}

impl Display for CreateFieldTraitsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateFieldTraitsError::DuplicateTraitName(v) => {
                writeln!(f, "Duplicate trait name: {v}")
            }
        }
    }
}

#[derive(Debug)]
pub struct DuplicateTraitName {
    trait_ident: syn::Ident,
    field_ident: syn::Ident,
}

impl Display for DuplicateTraitName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            r#"There are some fields with the same trait names. 
Macros will generate a trait definition for each 'field' and a trait name will be an ident 
from an attribute 'name' of a 'field' in PascalCase with a suffix 'Field'. 
Change your field names or use an explicit trait name with a 'trait_name' attribute of a 'field'. 
REMARK: fileds instead may have the same names, because each field generates a trait.
The error has been produced by a trait name '{}' on a field '{}'"#,
            self.trait_ident, self.field_ident
        )
    }
}

impl FieldTraits {
    pub fn create(attrs: Vec<FieldAttr>) -> Result<Self, CreateFieldTraitsError> {
        let fields_len = attrs.len();
        let mut by_trait_name = HashMap::new();
        let mut traits = Vec::with_capacity(fields_len);

        for field_attr in attrs {
            let field_trait = FieldTrait::create(field_attr);
            if by_trait_name
                .insert(field_trait.trait_ident.clone(), traits.len())
                .is_some()
            {
                return Err(CreateFieldTraitsError::DuplicateTraitName(
                    DuplicateTraitName {
                        trait_ident: field_trait.trait_ident,
                        field_ident: field_trait.field_ident,
                    },
                ));
            }
            traits.push(field_trait);
        }
        Ok(Self {
            traits,
            by_trait_name,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ FieldTrait> {
        self.traits.iter()
    }

    pub fn filter(&self, path: &EntryPath) -> Vec<&FieldTrait> {
        self.iter().filter(|v| v.is_match(path)).collect()
    }

    pub fn get(&self, ident: &Ident) -> Option<&FieldTrait> {
        self.by_trait_name
            .get(ident)
            .and_then(|&idx| self.traits.get(idx))
    }
}
