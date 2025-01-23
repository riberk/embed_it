pub mod attributes;
pub mod bool_like_enum;
pub mod pattern;
pub mod regex;

use std::{borrow::Cow, collections::HashSet};

use attributes::{
    dir::DirTrait,
    embed::{EmbedInput, GenerationSettings},
    entry::EntryStruct,
    field::FieldTrait,
    file::FileTrait,
};
use convert_case::{Case, Casing};
use darling::FromDeriveInput;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_quote, punctuated::Punctuated, token::PathSep, DeriveInput, Error, Ident, PathArguments,
    PathSegment,
};

use crate::{
    embedded_traits::{
        EmbeddedTrait, MakeEmbeddedTraitImplementationError, TraitAttr, EMBEDED_TRAITS,
    },
    fs::{Entry, EntryKind, FsInfo, ReadEntriesError},
    utils::{anymap::AnyMap, unique_names::UniqueNames},
};

pub(crate) fn impl_embed(input: DeriveInput) -> Result<proc_macro2::TokenStream, syn::Error> {
    let main_struct_ident = &input.ident;

    let input = EmbedInput::from_derive_input(&input)?;
    let settings = GenerationSettings::try_from(input)
        .map_err(|v| syn::Error::new_spanned(main_struct_ident, v))?;

    let mut context = GenerateContext::root(&settings)?;

    let impls = context
        .build_dir(&mut Vec::new(), &mut Vec::new())
        .map_err(|e| {
            Error::new_spanned(
                main_struct_ident,
                format!("Unable to build root struct: {e:#?}"),
            )
        })?;
    let dir_trait_definition = settings.dir.definition();
    let file_trait_definition = settings.file.definition();

    let field_traits_definition = settings
        .field_traits_definition()
        .map_err(|e| Error::new_spanned(main_struct_ident, e))?;
    let field_traits_implementation = context.field_traits_implementation();

    let embedded_traits_definition =
        generate_embedded_trait_definitions(&settings.dir, &settings.file, &settings.entry);
    let entry_implementation = settings.entry.implementation(&settings.dir, &settings.file);

    let dir_field_factory_definition = generate_factory_trait_definition(&settings.dir);
    let file_field_factory_definition = generate_factory_trait_definition(&settings.file);
    let stream = quote! {
        #embedded_traits_definition
        #entry_implementation
        #dir_trait_definition
        #file_trait_definition
        #dir_field_factory_definition
        #file_field_factory_definition
        #field_traits_definition

        #field_traits_implementation
        #impls
    };
    Ok(stream)
}

fn generate_embedded_trait_definitions(
    dir_attr: &DirTrait,
    file_attr: &FileTrait,
    entry_attr: &EntryStruct,
) -> proc_macro2::TokenStream {
    let mut stream = proc_macro2::TokenStream::new();

    let traits = dir_attr
        .embedded_traits()
        .chain(file_attr.embedded_traits())
        .map(|v| v.id())
        .collect::<HashSet<_>>()
        .into_iter()
        .map(|id| {
            EMBEDED_TRAITS
                .get(id)
                .unwrap_or_else(|| panic!("Unable to find trait with id `{id}`"))
        });
    let entry_path = entry_attr.ident();

    for embedded_trait in traits {
        stream.extend(embedded_trait.definition(entry_path));
    }

    stream
}

fn generate_factory_trait_definition(attr: &impl TraitAttr) -> proc_macro2::TokenStream {
    let trait_ident = attr.trait_ident();
    let ident = attr.field_factory_trait_ident();

    quote! {
        pub trait #ident {
            type Field;
            fn create<T: #trait_ident + ?Sized>(data: &T) -> Self::Field;
        }
    }
}

pub struct EntryTokens {
    pub struct_path: syn::Path,
    pub field_name: String,
    pub field_ident: syn::Ident,
    pub items: AnyMap,
    pub entry: Entry<FsInfo>,
}

pub struct IndexTokens {
    pub relative_path: String,
    pub struct_path: syn::Path,
    pub kind: EntryKind,
}

pub fn nested_module_path(nesting: usize) -> Punctuated<PathSegment, PathSep> {
    let mut new_segments = Punctuated::new();
    for _ in 0..nesting {
        new_segments.push(PathSegment {
            ident: Ident::new("super", Span::call_site()),
            arguments: PathArguments::None,
        });
    }
    new_segments
}

pub fn fix_path(path: &syn::Path, nesting: usize) -> Cow<'_, syn::Path> {
    if nesting == 0 {
        return Cow::Borrowed(path);
    }

    if path.leading_colon.is_some() {
        return Cow::Borrowed(path);
    }

    if path.segments.is_empty() {
        panic!("Empty path");
    }

    let first_segment = path.segments[0].ident.to_string();
    if first_segment == "crate" {
        return Cow::Borrowed(path);
    }

    let segments = if first_segment == "self" {
        path.segments.iter().skip(1)
    } else {
        #[allow(clippy::iter_skip_zero)]
        path.segments.iter().skip(0)
    };

    let mut new_segments = nested_module_path(nesting);
    new_segments.extend(segments.cloned());

    let path = syn::Path {
        leading_colon: None,
        segments: new_segments,
    };

    Cow::Owned(path)
}

/// A context of an entry generation
pub struct GenerateContext<'a> {
    /// A nesting level of the entry
    pub level: usize,

    /// An information about the file system entry
    pub entry: Entry<FsInfo>,

    /// The unique names of the module/impl items
    pub names: UniqueNames,

    /// The string representation of [`Self::struct_ident``]
    pub struct_name: String,

    /// The identifier of the struct (PascalCase)
    pub struct_ident: syn::Ident,

    /// The string representation of [`Self::mod_ident``]
    pub mod_name: String,

    /// The identifier of the module and field (snake_case)
    pub mod_ident: syn::Ident,

    /// The path of the `Entry` struct (including `super::super::...::$ident`)
    pub entry_path: syn::Path,

    pub settings: &'a GenerationSettings,

    pub fields: Vec<&'a FieldTrait>,

    pub items: AnyMap,

    /// The parents of the entry from the root to the direct
    pub parents: Vec<ParentTokens>,
}

/// Parent of the entry
#[derive(Debug, Clone)]
pub struct ParentTokens {
    /// The identifier of the parent struct
    pub struct_ident: syn::Ident,
}

impl<'a> GenerateContext<'a> {
    fn entry_trait(&self) -> Entry<&DirTrait, &FileTrait> {
        self.settings.trait_for(self.entry.kind())
    }

    /// Creates the root-level context
    fn root(settings: &'a GenerationSettings) -> Result<Self, syn::Error> {
        let entry = Entry::root(&settings.root).map_err(|e| {
            Error::new_spanned(
                &settings.main_struct_ident,
                format!(
                    "Unable to read directory '{:?}' information: {e:?}",
                    settings.root
                ),
            )
        })?;

        let mod_name = settings.main_struct_ident.to_string().to_case(Case::Snake);
        let mod_ident = Ident::new(&mod_name, Span::call_site());

        let entry_path = Self::make_nested_path(0, settings.entry.ident().to_owned());
        Ok(Self {
            level: 0,
            fields: settings.dir.fields().filter(entry.path()),
            entry,
            names: UniqueNames::default(),
            struct_name: String::new(),
            struct_ident: settings.main_struct_ident.clone(),
            mod_name,
            mod_ident,
            entry_path,
            settings,
            items: Default::default(),
            parents: Default::default(),
        })
    }

    /// Creates a context for a child of the current
    fn child(&self, entry: Entry<FsInfo>) -> Self {
        let struct_name = entry.path().ident.to_case(Case::Pascal);
        let struct_ident = Ident::new_raw(&struct_name, Span::call_site());
        let mod_name = entry.path().ident.to_case(Case::Snake);
        let mod_ident = Ident::new_raw(&mod_name, Span::call_site());
        let level = self.level + 1;
        let entry_path = Self::make_nested_path(level, self.settings.entry.ident().to_owned());
        let mut parents = self.parents.clone();
        parents.push(ParentTokens {
            struct_ident: self.struct_ident.clone(),
        });
        Self {
            level,
            fields: self
                .settings
                .trait_for(entry.kind())
                .map(|v| v.fields(), |v| v.fields())
                .value()
                .filter(entry.path()),
            entry,
            names: UniqueNames::default(),
            struct_name,
            struct_ident,
            mod_name,
            mod_ident,
            entry_path,
            settings: self.settings,
            items: Default::default(),
            parents,
        }
    }

    fn field_traits_implementation(&self) -> proc_macro2::TokenStream {
        self.fields
            .iter()
            .filter(|t| t.is_match(self.entry.path()))
            .fold(proc_macro2::TokenStream::new(), |mut accum, field| {
                accum.extend(field.implementation(self));
                accum
            })
    }

    fn build(
        mut self,
        parent_entries: &mut Vec<EntryTokens>,
        parent_index: &mut Vec<IndexTokens>,
    ) -> Result<proc_macro2::TokenStream, BuildStreamError> {
        let mut entries = Vec::new();
        let mut index = Vec::new();

        let stream = match self.entry.kind() {
            EntryKind::File => self
                .build_file(&entries, &index)
                .map_err(BuildStreamError::File)?,
            EntryKind::Dir => self
                .build_dir(&mut entries, &mut index)
                .map_err(BuildStreamError::Dir)?,
        };

        let struct_ident = &self.struct_ident;
        let mod_ident = &self.mod_ident;

        let traits = self.field_traits_implementation();
        let stream = quote! {
            pub mod #mod_ident {
                #[derive(Clone, Copy, PartialEq, Eq, Hash)]
                pub struct #struct_ident;

                #stream
                #traits
            }
        };

        let index_relative_path = &self.entry.path().file_name;
        parent_index.extend(index.into_iter().map(|mut i| {
            let prev_path = i.struct_path;
            i.struct_path = parse_quote!(#mod_ident::#prev_path);
            i.relative_path = format!("{index_relative_path}/{}", i.relative_path);
            i
        }));

        let struct_path: syn::Path = parse_quote!(#mod_ident::#struct_ident);
        parent_index.push(IndexTokens {
            relative_path: index_relative_path.clone(),
            struct_path: struct_path.clone(),
            kind: self.entry.kind(),
        });

        parent_entries.push(EntryTokens {
            struct_path: struct_path.clone(),
            field_name: self.mod_name,
            field_ident: self.mod_ident,
            items: self.items,
            entry: self.entry,
        });

        Ok(stream)
    }

    pub fn make_nested_path(nesting: usize, ident: Ident) -> syn::Path {
        let mut path = nested_module_path(nesting);
        path.push(PathSegment {
            ident,
            arguments: PathArguments::None,
        });
        syn::Path {
            leading_colon: None,
            segments: path,
        }
    }

    pub fn make_level_path(&self, ident: Ident) -> syn::Path {
        Self::make_nested_path(self.level, ident)
    }

    fn build_dir(
        &mut self,
        entries: &mut Vec<EntryTokens>,
        index: &mut Vec<IndexTokens>,
    ) -> Result<proc_macro2::TokenStream, BuildDirError> {
        let children = Entry::read(
            self.entry.path().origin_path(),
            &self.settings.root,
            self.settings.with_extension,
            &mut self.names,
        )
        .map_err(BuildDirError::ReadEntries)?;
        let mut modules = proc_macro2::TokenStream::new();
        for entry in children {
            let child = self.child(entry);
            modules.extend(child.build(entries, index));
        }

        let impl_stream = self
            .settings
            .dir
            .implementation_stream(self, entries, index)
            .map_err(BuildDirError::MakeEmbeddedTraitImplementation)?;
        let stream = quote! {
            #impl_stream
            #modules
        };

        Ok(stream)
    }

    fn build_file(
        &mut self,
        entries: &[EntryTokens],
        index: &[IndexTokens],
    ) -> Result<proc_macro2::TokenStream, BuildFileError> {
        self.settings
            .file
            .implementation_stream(self, entries, index)
            .map_err(BuildFileError::MakeEmbeddedTraitImplementation)
    }

    pub fn is_trait_implemented_for(
        &self,
        kind: EntryKind,
        expected: &'static impl EmbeddedTrait,
    ) -> bool {
        match kind {
            EntryKind::Dir => self.settings.dir.is_trait_implemented(expected),
            EntryKind::File => self.settings.file.is_trait_implemented(expected),
        }
    }
}

#[derive(Debug)]
pub enum BuildStreamError {
    Dir(#[allow(dead_code)] BuildDirError),
    File(#[allow(dead_code)] BuildFileError),
}

#[derive(Debug)]
pub enum BuildDirError {
    ReadEntries(#[allow(dead_code)] ReadEntriesError),
    MakeEmbeddedTraitImplementation(#[allow(dead_code)] MakeEmbeddedTraitImplementationError),
}

#[derive(Debug)]
pub enum BuildFileError {
    MakeEmbeddedTraitImplementation(#[allow(dead_code)] MakeEmbeddedTraitImplementationError),
}

#[cfg(test)]
mod tests {

    use crate::{
        fn_name,
        test_helpers::{
            create_dir_all, create_file, derive_input, remove_and_create_dir_all, tests_dir, Print,
        },
    };

    use super::{attributes::embed::EmbedInput, fix_path, impl_embed, GenerationSettings};
    use proc_macro2::Span;
    use quote::quote;
    use syn::{punctuated::Punctuated, Ident};

    #[test]
    fn check_macros_simple() {
        let current_dir = tests_dir().join(fn_name!());
        remove_and_create_dir_all(&current_dir);
        create_file(current_dir.join("file1.txt"), b"hello");

        let subdir1 = current_dir.join("subdir1");
        create_dir_all(&subdir1);
        create_file(subdir1.join("file1.txt"), b"hello");

        let path = current_dir.to_str().unwrap();

        let input = derive_input(quote! {
            #[derive(Embed)]
            #[embed(path = #path)]
            pub struct Assets;
        });

        impl_embed(input).print_to_std_out();
    }

    #[test]
    fn check_macros() {
        let current_dir = tests_dir().join(fn_name!());
        remove_and_create_dir_all(&current_dir);
        create_file(current_dir.join("file1.txt"), b"hello");
        create_file(current_dir.join("file2.txt"), b"world");

        let subdir1 = current_dir.join("subdir1");
        create_dir_all(&subdir1);
        create_file(subdir1.join("file1.txt"), b"hello");
        create_file(subdir1.join("file2.txt"), b"world");

        let subdir2 = current_dir.join("subdir2");
        create_dir_all(&subdir2);
        create_file(subdir2.join("file1.txt"), b"hello");
        create_file(subdir2.join("file2.txt"), b"world");

        let path = current_dir.to_str().unwrap();

        let input = derive_input(quote! {
            #[derive(Embed)]
            #[embed(
                path = #path,
                file(
                    field(pattern = "*.txt", factory = "Handle", name = "handle"),
                ),
            )]
            pub struct Assets;
        });

        impl_embed(input).print_to_std_out();
    }

    #[test]
    fn check_macros_the_same_normalized_names() {
        let current_dir = tests_dir().join(fn_name!());
        remove_and_create_dir_all(&current_dir);
        create_dir_all(current_dir.join("subdir.txt"));
        create_dir_all(current_dir.join("subdir_txt"));
        create_dir_all(current_dir.join("subdir+txt"));
        create_file(current_dir.join("subdir)txt"), b"hello");
        create_file(current_dir.join("subdir=txt"), b"hello");
        create_file(current_dir.join("subdir-txt"), b"hello");

        let path = current_dir.to_str().unwrap();

        let input = derive_input(quote! {
            #[derive(Embed)]
            #[embed(path = #path, with_extension = true)]
            pub struct Assets;
        });

        impl_embed(input).print_to_std_out();
    }

    #[test]
    fn all_attributes() {
        let current_dir = tests_dir().join(fn_name!());
        remove_and_create_dir_all(&current_dir);
        create_dir_all(current_dir.join("subdir.txt"));
        create_dir_all(current_dir.join("subdir_txt"));
        create_dir_all(current_dir.join("subdir+txt"));
        create_file(current_dir.join("subdir)txt"), b"hello");
        create_file(current_dir.join("subdir=txt"), b"hello");
        create_file(current_dir.join("subdir-txt"), b"hello");

        let path = current_dir.to_str().unwrap();

        let input = derive_input(quote! {
            #[derive(Embed)]
            #[embed(
                path = #path,
                dir(
                    trait_name = AssetsDir,
                    field_factory_trait_name = AssetsDirFieldFactory,
                    derive(Path),
                    derive(Entries),
                    derive(Index),
                    derive(Meta),
                    derive(Debug),
                    field(
                        name = children,
                        trait_name = AssetsChildrenField,
                        factory = self::Children,
                        pattern = "?*",
                        regex = ".+",
                    ),
                    field(
                        name = root_children,
                        trait_name = AssetsRootChildrenField,
                        factory = ::other::Children,
                        regex = "",
                    ),
                    mark(ChildOf),
                ),
                file(
                    trait_name = AssetsFile,
                    field_factory_trait_name = AssetsFileFieldFactory,
                    derive(Path),
                    derive(Meta),
                    derive(Content),
                    derive(Debug),
                    field(
                        name = as_str,
                        trait_name = AssetsAsStrField,
                        factory = crate::AsStr,
                        pattern = "*.txt",
                        regex = ".+",
                    ),
                    mark(ChildOf),
                ),
                entry(
                    struct_name = AssetsEntry,
                ),
                with_extension = true,


            )]
            pub struct Assets;
        });

        impl_embed(input).print_to_std_out();
    }

    #[test]
    fn same_field_traits() {
        let current_dir = tests_dir().join(fn_name!());
        let path = current_dir.to_str().unwrap();
        remove_and_create_dir_all(&current_dir);

        let input = derive_input(quote! {
            #[derive(Embed)]
            #[embed(
                path = #path,
                file(
                    field(name = as_str, trait_name = AssetsAsStrField, factory = AsStr),
                    field(name = as_str2, trait_name = AssetsAsStrField, factory = AsStr),
                )
            )]
            pub struct Assets;
        });

        let err = format!("{:?}", impl_embed(input).unwrap_err());
        assert!(
            err.contains("AssetsAsStrField"),
            "Unable to find a trait name in a error string: '{}'",
            err
        );
        assert!(
            err.contains("as_str2"),
            "Unable to find a method name in a error string: '{}'",
            err
        );
    }

    #[test]
    fn generation_settings_creation_error_path_does_not_exist() {
        let dir_name = fn_name!();
        let current_dir = tests_dir().join(&dir_name);

        let path_str = current_dir.to_str().unwrap();
        let input = EmbedInput {
            ident: Ident::new("sss", Span::call_site()),
            path: path_str.to_owned(),
            with_extension: Default::default(),
            support_alt_separator: Default::default(),
            dir: Default::default(),
            file: Default::default(),
            entry: Default::default(),
        };
        let err = GenerationSettings::try_from(input).unwrap_err();
        let err_str = format!("{err:?}");
        assert!(err_str.contains(&dir_name));
    }

    #[test]
    #[should_panic(expected = "Empty path")]
    fn fix_path_panics_if_empty() {
        let path = syn::Path {
            leading_colon: None,
            segments: Punctuated::new(),
        };
        fix_path(&path, 10);
    }

    #[test]
    #[cfg(all(
        feature = "md5",
        feature = "sha1",
        feature = "sha2",
        feature = "sha3",
        feature = "blake3"
    ))]
    fn hashes() {
        let current_dir = tests_dir().join(fn_name!());
        remove_and_create_dir_all(&current_dir);
        create_file(current_dir.join("hello.txt"), b"hello");
        create_file(current_dir.join("world.txt"), b"world");
        create_file(current_dir.join("one.txt"), b"one");

        let subdir1 = current_dir.join("one_txt");
        create_dir_all(&subdir1);
        create_file(subdir1.join("hello"), b"hello");
        create_file(subdir1.join("world"), b"world");

        let path = current_dir.to_str().unwrap();

        let input = derive_input(quote! {
            #[derive(Embed)]
            #[embed(
                path = #path,
                dir(
                    derive(Md5),
                    derive(Sha1),
                    derive(Sha2_224),
                    derive(Sha2_256),
                    derive(Sha2_384),
                    derive(Sha2_384),
                    derive(Sha2_512),
                    derive(Sha3_224),
                    derive(Sha3_256),
                    derive(Sha3_384),
                    derive(Sha3_384),
                    derive(Sha3_512),
                    derive(Blake3),
                ),
                file(
                    derive(Md5),
                    derive(Sha1),
                    derive(Sha2_224),
                    derive(Sha2_256),
                    derive(Sha2_384),
                    derive(Sha2_384),
                    derive(Sha2_512),
                    derive(Sha3_224),
                    derive(Sha3_256),
                    derive(Sha3_384),
                    derive(Sha3_384),
                    derive(Sha3_512),
                    derive(Blake3),
                ),
            )]
            pub struct Assets;
        });

        impl_embed(input).unwrap();
    }

    #[test]
    #[cfg(feature = "md5")]
    fn hashes_only_dir() {
        let current_dir = tests_dir().join(fn_name!());
        remove_and_create_dir_all(&current_dir);
        create_file(current_dir.join("hello.txt"), b"hello");
        create_file(current_dir.join("world.txt"), b"world");
        create_file(current_dir.join("one.txt"), b"one");

        let subdir1 = current_dir.join("one_txt");
        create_dir_all(&subdir1);
        create_file(subdir1.join("hello"), b"hello");
        create_file(subdir1.join("world"), b"world");

        let path = current_dir.to_str().unwrap();

        let input = derive_input(quote! {
            #[derive(Embed)]
            #[embed(
                path = #path,
                dir(
                    derive(Md5),
                ),
            )]
            pub struct Assets;
        });

        impl_embed(input).print_to_std_out();
    }

    #[test]
    #[cfg(feature = "md5")]
    fn hashes_only_file() {
        let current_dir = tests_dir().join(fn_name!());
        remove_and_create_dir_all(&current_dir);
        create_file(current_dir.join("hello.txt"), b"hello");
        create_file(current_dir.join("world.txt"), b"world");
        create_file(current_dir.join("one.txt"), b"one");

        let subdir1 = current_dir.join("one_txt");
        create_dir_all(&subdir1);
        create_file(subdir1.join("hello"), b"hello");
        create_file(subdir1.join("world"), b"world");

        let path = current_dir.to_str().unwrap();

        let input = derive_input(quote! {
            #[derive(Embed)]
            #[embed(
                path = #path,
                file(derive(Md5)),
            )]
            pub struct Assets;
        });

        impl_embed(input).print_to_std_out();
    }

    #[test]
    #[cfg(not(feature = "md5"))]
    fn hash_with_disabled_feature() {
        let current_dir = tests_dir().join(fn_name!());
        remove_and_create_dir_all(&current_dir);
        create_file(current_dir.join("hello.txt"), b"hello");

        let path = current_dir.to_str().unwrap();

        let input = derive_input(quote! {
            #[derive(Embed)]
            #[embed(
                path = #path,
                file(derive(Md5)),
            )]
            pub struct Assets;
        });

        let err = impl_embed(input).unwrap_err().to_string();

        assert_eq!(&err, "Unable to parse `file` attribute: Unable to resolve embedded trait: Feature 'md5' must be enabled to use 'Hash(md5)'");
    }
}
