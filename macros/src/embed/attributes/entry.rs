use std::collections::HashMap;

use darling::FromMeta;
use proc_macro2::Span;
use quote::quote;
use syn::Ident;

use crate::embedded_traits::TraitAttr;

use super::{dir::DirTrait, file::FileTrait};

#[derive(Debug, Default, FromMeta)]
pub struct EntryAttr {
    #[darling(default)]
    struct_name: Option<Ident>,
}

#[derive(Debug)]
pub struct EntryStruct {
    struct_name: Ident,
}

impl From<EntryAttr> for EntryStruct {
    fn from(value: EntryAttr) -> Self {
        Self {
            struct_name: value
                .struct_name
                .unwrap_or_else(|| Ident::new("Entry", Span::call_site())),
        }
    }
}

impl EntryStruct {
    pub fn ident(&self) -> &Ident {
        &self.struct_name
    }

    pub fn implementation(&self, dir: &DirTrait, file: &FileTrait) -> proc_macro2::TokenStream {
        let dir_traits = dir
            .embedded_traits()
            .map(|v| (v.id(), v))
            .collect::<HashMap<_, _>>();
        let file_traits = file
            .embedded_traits()
            .map(|v| (v.id(), v))
            .collect::<HashMap<_, _>>();
        let ident = self.ident();
        let dir_trait = dir.trait_ident();
        let file_trait = file.trait_ident();
        let mut stream = quote! {
            pub enum #ident {
                Dir(&'static dyn #dir_trait),
                File(&'static dyn #file_trait),
            }

            impl #ident {
                /// If it's a dir returns Some, else None
                pub fn dir(&self) -> Option<&'static dyn #dir_trait> {
                    match self {
                        Self::File(_) => None,
                        Self::Dir(d) => Some(*d),
                    }
                }

                /// If it's a file returns Some, else None
                pub fn file(&self) -> Option<&'static dyn #file_trait> {
                    match self {
                        Self::File(f) => Some(*f),
                        Self::Dir(_) => None,
                    }
                }
            }

            impl From<&'static dyn #file_trait> for #ident {
                fn from(value: &'static dyn #file_trait) -> Self {
                    Self::File(value)
                }
            }

            impl From<&'static dyn #dir_trait> for #ident {
                fn from(value: &'static dyn #dir_trait) -> Self {
                    Self::Dir(value)
                }
            }
        };

        for (dir_trait_id, dir_trait) in dir_traits {
            if file_traits.contains_key(dir_trait_id) {
                let trait_path = dir_trait.path(0);
                let impl_body = dir_trait.entry_impl_body();
                stream.extend(quote! {
                    impl #trait_path for #ident {
                        #impl_body
                    }
                });
            }
        }

        stream
    }
}
