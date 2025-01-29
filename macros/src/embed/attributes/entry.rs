use darling::FromMeta;
use quote::quote;
use syn::{parse_quote, punctuated::Punctuated, Ident, Token, TypeParamBound};

use crate::embedded_traits::{debug::DebugTrait, TraitAttr};

use super::{dir::DirTrait, file::FileTrait};

#[derive(Debug, Default, FromMeta)]
pub struct EntryAttr {
    #[darling(default)]
    dir_trait_name: Option<Ident>,

    #[darling(default)]
    file_trait_name: Option<Ident>,

    #[darling(default)]
    dir_struct_name: Option<Ident>,

    #[darling(default)]
    file_struct_name: Option<Ident>,
}

#[derive(Debug)]
pub struct EntryStruct {
    dir_trait_name: Ident,
    file_trait_name: Ident,
    dir_struct_name: Ident,
    file_struct_name: Ident,
}

impl From<EntryAttr> for EntryStruct {
    fn from(value: EntryAttr) -> Self {
        Self {
            dir_trait_name: value
                .dir_trait_name
                .unwrap_or_else(|| parse_quote!(EntryDir)),
            file_trait_name: value
                .file_trait_name
                .unwrap_or_else(|| parse_quote!(EntryFile)),
            dir_struct_name: value
                .dir_struct_name
                .unwrap_or_else(|| parse_quote!(DynDir)),
            file_struct_name: value
                .file_struct_name
                .unwrap_or_else(|| parse_quote!(DynFile)),
        }
    }
}

impl EntryStruct {
    pub fn dir_trait_ident(&self) -> &Ident {
        &self.dir_trait_name
    }

    pub fn file_trait_ident(&self) -> &Ident {
        &self.file_trait_name
    }

    pub fn dir_struct_ident(&self) -> &Ident {
        &self.dir_struct_name
    }

    pub fn file_struct_ident(&self) -> &Ident {
        &self.file_struct_name
    }

    pub fn implementation(&self, dir: &DirTrait, file: &FileTrait) -> proc_macro2::TokenStream {
        let dir_trait = dir.trait_ident();
        let file_trait = file.trait_ident();

        let entry_dir_trait = &self.dir_trait_ident();
        let entry_file_trait = &self.file_trait_ident();

        let entry_dir_struct = &self.dir_struct_ident();
        let entry_file_struct = &self.file_struct_ident();

        let mut dir_bounds = Punctuated::<TypeParamBound, Token![+]>::new();
        dir_bounds.push(parse_quote!(#dir_trait));
        for b in dir.fields().iter().filter_map(|f| f.global_bound()) {
            dir_bounds.push(b);
        }

        let mut file_bounds = Punctuated::<TypeParamBound, Token![+]>::new();
        file_bounds.push(parse_quote!(#file_trait));
        for b in file.fields().iter().filter_map(|f| f.global_bound()) {
            file_bounds.push(b);
        }

        let mut file_derive = Punctuated::<syn::Path, Token![,]>::new();
        file_derive.push(parse_quote!(Clone));
        file_derive.push(parse_quote!(Copy));
        if file.is_trait_implemented(&DebugTrait) {
            file_derive.push(parse_quote!(Debug));
        }

        let mut dir_derive = Punctuated::<syn::Path, Token![,]>::new();
        dir_derive.push(parse_quote!(Clone));
        dir_derive.push(parse_quote!(Copy));
        if dir.is_trait_implemented(&DebugTrait) {
            dir_derive.push(parse_quote!(Debug));
        }

        let stream = quote! {
            pub trait #entry_dir_trait: #dir_bounds {}
            pub trait #entry_file_trait: #file_bounds {}

            #[derive(#dir_derive)]
            pub struct #entry_dir_struct(&'static dyn #entry_dir_trait);

            #[automatically_derived]
            impl #entry_dir_struct {
                fn into_dir(self) -> &'static dyn #entry_dir_trait {
                    self.0
                }
            }

            #[automatically_derived]
            impl ::std::ops::Deref for #entry_dir_struct {
                type Target = dyn #entry_dir_trait;

                fn deref(&self) -> &Self::Target {
                    self.0
                }
            }

            #[derive(#file_derive)]
            pub struct #entry_file_struct(&'static dyn #entry_file_trait);

            #[automatically_derived]
            impl ::std::ops::Deref for #entry_file_struct {
                type Target = dyn #entry_file_trait;

                fn deref(&self) -> &Self::Target {
                    self.0
                }
            }

            #[automatically_derived]
            impl #entry_file_struct {
                fn into_file(self) -> &'static dyn #entry_file_trait {
                    self.0
                }
            }
        };

        stream
    }
}
