use std::collections::HashSet;

use convert_case::{Case, Casing};
use darling::FromMeta;
use proc_macro2::Span;
use quote::quote;
use syn::Ident;

use crate::{
    embed::{fix_path, pattern::EntryPattern, regex::EntryRegex, GenerateContext},
    embedded_traits::TraitAttr,
    fs::{Entry, EntryKind},
};

use super::{dir::DirAttr, file::FileAttr};

#[derive(Debug, FromMeta)]
pub struct FieldAttr {
    regex: Option<EntryRegex>,

    pattern: Option<EntryPattern>,

    #[darling(default)]
    target: EntryKind,

    factory: syn::Path,

    name: syn::Ident,

    trait_name: Option<syn::Ident>,
}

#[derive(Debug)]
pub struct FieldTrait {
    field_ident: syn::Ident,
    trait_ident: syn::Ident,
    bound_ident: syn::Ident,
    factory_trait: syn::Ident,
    factory: syn::Path,
    regex: Option<EntryRegex>,
    pattern: Option<EntryPattern>,

    target: EntryKind,
}

impl FieldTrait {
    pub fn create(field_attr: FieldAttr, dir_attr: &DirAttr, file_attr: &FileAttr) -> Self {
        let trait_ident = field_attr.trait_name.unwrap_or_else(|| {
            let name = format!("{}Field", field_attr.name.to_string().to_case(Case::Pascal));
            Ident::new_raw(&name, Span::call_site())
        });

        let (bound_ident, factory_trait) = match field_attr.target {
            EntryKind::Dir => (dir_attr.trait_ident(), dir_attr.field_factory_trait_ident()),
            EntryKind::File => (
                file_attr.trait_ident(),
                file_attr.field_factory_trait_ident(),
            ),
        };

        Self {
            field_ident: field_attr.name,
            trait_ident,
            bound_ident: bound_ident.into_owned(),
            factory_trait: factory_trait.into_owned(),
            regex: field_attr.regex,
            pattern: field_attr.pattern,
            target: field_attr.target,
            factory: field_attr.factory,
        }
    }

    pub fn is_match(&self, entry: &Entry) -> bool {
        entry.kind() == self.target
            && self
                .regex
                .as_ref()
                .map(|v| v.is_match(entry.path()))
                .unwrap_or(true)
            && self
                .pattern
                .as_ref()
                .map(|v| v.is_match(entry.path()))
                .unwrap_or(true)
    }

    pub fn definition(&self) -> proc_macro2::TokenStream {
        let FieldTrait {
            field_ident,
            trait_ident,
            bound_ident,
            factory_trait,
            factory,
            ..
        } = self;

        quote! {
            pub trait #trait_ident: #bound_ident {
                fn #field_ident(&self) -> &'static <#factory as #factory_trait>::Field;
            }
        }
    }

    pub fn implementation(&self, ctx: &GenerateContext<'_>) -> proc_macro2::TokenStream {
        if !self.is_match(&ctx.entry) {
            return Default::default();
        }

        let FieldTrait {
            field_ident,
            trait_ident,
            factory_trait,
            factory,
            ..
        } = self;

        let struct_ident = &ctx.struct_ident;
        let factory = fix_path(factory, ctx.level);
        let trait_path = ctx.make_level_path(trait_ident.clone());
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
}

#[derive(Debug)]
pub struct FieldTraits(Vec<FieldTrait>);

impl FieldTraits {
    pub fn try_from_attrs(
        attrs: Vec<FieldAttr>,
        dir: &DirAttr,
        file: &FileAttr,
        main_struct_ident: &Ident,
    ) -> Result<Self, syn::Error> {
        let fields_len = attrs.len();
        let mut trait_names = HashSet::new();
        let mut traits = Vec::with_capacity(fields_len);

        for field_attr in attrs {
            let field_trait = FieldTrait::create(field_attr, dir, file);
            if !trait_names.insert(field_trait.trait_ident.clone()) {
                return Err(syn::Error::new_spanned(
                    main_struct_ident,
                    format!(
                        r#"There are some fields with the same trait names. 
Macros will generate a trait definition for each 'field' and a trait name will be an ident 
from an attribute 'name' of a 'field' in PascalCase with a suffix 'Field'. 
Change your field names or use an explicit trait name with a 'trait_name' attribute of a 'field'. 
REMARK: fileds instead may have the same names, because each field generates a trait.
The error has been produced by a trait name '{}' on a field '{}'"#,
                        field_trait.trait_ident, field_trait.field_ident
                    ),
                ));
            }
            traits.push(field_trait);
        }
        Ok(Self(traits))
    }

    pub fn definitions(&self) -> proc_macro2::TokenStream {
        let mut stream = proc_macro2::TokenStream::new();

        for field in &self.0 {
            stream.extend(field.definition());
        }
        quote! {
            #stream
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ FieldTrait> {
        self.0.iter()
    }
}
