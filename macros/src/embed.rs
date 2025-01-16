use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    path::Path,
};

use convert_case::{Case, Casing};
use darling::{FromDeriveInput, FromMeta};
use proc_macro2::Span;
use quote::quote;
use strum::VariantArray;
use syn::{
    parse_quote, punctuated::Punctuated, token::PathSep, DeriveInput, Error, Ident, PathArguments,
    PathSegment,
};

use crate::{
    embedded_traits::{
        content::ContentTrait, debug::DebugTrait, entries::EntriesTrait, index::IndexTrait,
        meta::MetaTrait, path::PathTrait, EmbeddedTrait, TraitAttr, EMBEDED_TRAITS,
    },
    fs::{expand_and_canonicalize, get_env, Entry, EntryKind, EntryPath, ReadEntriesError},
    unique_names::UniqueNames,
};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(embed), supports(struct_unit))]
struct EmbedInput {
    path: String,

    with_extensions: Option<bool>,

    #[darling(default, multiple, rename = "field")]
    fields: Vec<FieldAttr>,

    #[darling(default)]
    dir: DirAttr,

    #[darling(default)]
    file: FileAttr,

    #[darling(default)]
    entry: EntryAttr,
}

#[derive(Debug, FromMeta)]
pub struct DirAttr {
    #[darling(default)]
    trait_name: Option<Ident>,

    #[darling(default = default_dir_traits, multiple, rename = "derive")]
    traits: Vec<DirTrait>,

    #[darling(default)]
    field_factory_trait_name: Option<Ident>,
}

impl Default for DirAttr {
    fn default() -> Self {
        Self {
            trait_name: None,
            traits: default_dir_traits(),
            field_factory_trait_name: None,
        }
    }
}

#[derive(Debug, FromMeta, strum_macros::VariantArray, Clone, Copy, PartialEq, Eq)]
#[darling(rename_all = "PascalCase")]
pub enum DirTrait {
    Path,
    Entries,
    Index,
    Meta,
    Debug,
}

impl DirTrait {
    fn as_embedded_trait(&self) -> &'static dyn EmbeddedTrait {
        match self {
            DirTrait::Path => &PathTrait,
            DirTrait::Entries => &EntriesTrait,
            DirTrait::Index => &IndexTrait,
            DirTrait::Meta => &MetaTrait,
            DirTrait::Debug => &DebugTrait,
        }
    }
}

#[derive(Debug, FromMeta, strum_macros::VariantArray, Clone, Copy, PartialEq, Eq)]
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

fn default_dir_traits() -> Vec<DirTrait> {
    DirTrait::VARIANTS.to_vec()
}

fn default_file_traits() -> Vec<FileTrait> {
    FileTrait::VARIANTS.to_vec()
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

impl TraitAttr for DirAttr {
    const DEFAULT_TRAIT_NAME: &str = "Dir";
    const DEFAULT_FIELD_FACTORY_TRAIT_NAME: &str = "DirFieldFactory";

    fn trait_name(&self) -> Option<&Ident> {
        self.trait_name.as_ref()
    }

    fn field_factory_trait_name(&self) -> Option<&Ident> {
        self.field_factory_trait_name.as_ref()
    }

    fn traits(&self) -> impl Iterator<Item = &'static dyn EmbeddedTrait> {
        self.traits.iter().map(|v| v.as_embedded_trait())
    }

    fn struct_impl(
        &self,
        ctx: &GenerateContext<'_>,
        entries: &[EntryTokens],
    ) -> proc_macro2::TokenStream {
        let struct_ident = &ctx.struct_ident;
        let methods = entries.iter().fold(quote! {}, |mut acc, entry| {
            let EntryTokens {
                struct_path,
                field_ident,
                ..
            } = entry;
            acc.extend(quote! {
                pub fn #field_ident(&self) -> &'static #struct_path {
                    &#struct_path
                }
            });
            acc
        });

        quote! {
            impl #struct_ident {
                #methods
            }
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

#[derive(Debug, Default, FromMeta)]
pub struct EntryAttr {
    #[darling(default)]
    struct_name: Option<Ident>,
}

impl EntryAttr {
    pub fn ident(&self) -> Cow<'_, Ident> {
        match &self.struct_name {
            Some(ident) => Cow::Borrowed(ident),
            None => Cow::Owned(Ident::new("Entry", Span::call_site())),
        }
    }

    pub fn implementation(
        &self,
        dir_attr: &DirAttr,
        file_attr: &FileAttr,
    ) -> proc_macro2::TokenStream {
        let dir_traits = dir_attr
            .traits()
            .map(|v| (v.id(), v))
            .collect::<HashMap<_, _>>();
        let file_traits = file_attr
            .traits()
            .map(|v| (v.id(), v))
            .collect::<HashMap<_, _>>();
        let ident = self.ident();
        let dir_trait = dir_attr.trait_ident();
        let file_trait = file_attr.trait_ident();
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

#[derive(Debug, FromMeta)]
pub struct FieldAttr {
    regex: Option<regex::EntryRegex>,

    pattern: Option<pattern::EntryPattern>,

    #[darling(default)]
    target: EntryKind,

    factory: syn::Path,

    name: syn::Ident,

    trait_name: Option<syn::Ident>,
}

mod pattern {
    use darling::FromMeta;
    use glob::Pattern;

    use super::EntryPath;

    #[derive(Debug, Clone)]
    pub struct EntryPattern(Pattern);

    impl FromMeta for EntryPattern {
        fn from_string(value: &str) -> darling::Result<Self> {
            Pattern::new(value)
                .map_err(|e| {
                    darling::Error::custom(format!(
                        "'{}' is not a valid glob pattern: {:#?}",
                        value, e
                    ))
                })
                .map(Self)
        }
    }

    impl EntryPattern {
        pub fn is_match(&self, path: &EntryPath) -> bool {
            self.0.matches_path(path.relative_path())
        }
    }
}

mod regex {
    use darling::FromMeta;
    use regex::Regex;

    use super::EntryPath;

    #[derive(Debug, Clone)]
    pub struct EntryRegex(Regex);

    impl FromMeta for EntryRegex {
        fn from_string(value: &str) -> darling::Result<Self> {
            Regex::new(value)
                .map_err(|e| {
                    darling::Error::custom(format!("'{}' is not a valid regex: {:#?}", value, e))
                })
                .map(Self)
        }
    }

    impl EntryRegex {
        pub fn is_match(&self, path: &EntryPath) -> bool {
            self.0.is_match(&path.relative)
        }
    }
}

pub(crate) fn impl_embed(input: DeriveInput) -> Result<proc_macro2::TokenStream, syn::Error> {
    let main_struct_ident = &input.ident;

    let input = EmbedInput::from_derive_input(&input)?;

    let root = expand_and_canonicalize(&input.path, get_env).map_err(|e| {
        Error::new_spanned(
            main_struct_ident,
            format!(
                "Unable to expand and canonicalize path '{}': {:?}",
                &input.path, e
            ),
        )
    })?;

    let with_extensions = input.with_extensions.unwrap_or_default();
    let fields_len = input.fields.len();
    let mut trait_names = HashSet::new();
    let mut traits = Vec::with_capacity(fields_len);

    for field_attr in input.fields {
        let field_trait = FieldTrait::create(field_attr, &input.dir, &input.file);
        if !trait_names.insert(field_trait.trait_ident.clone()) {
            return Err(Error::new_spanned(
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

    let mut context = GenerateContext::root(
        main_struct_ident,
        &root,
        &traits,
        with_extensions,
        &input.dir,
        &input.file,
        &input.entry,
    )?;

    let impls = context
        .build_dir(&mut Vec::new(), &mut Vec::new())
        .map_err(|e| {
            Error::new_spanned(
                main_struct_ident,
                format!("Unable to build root struct: {e:#?}"),
            )
        })?;
    let dir_trait_definition = input.dir.definition();
    let file_trait_definition = input.file.definition();

    let field_traits_definition = generate_field_trait_definitions(&traits);
    let field_traits_implementation = context.field_traits_implementation();

    let embedded_traits_definition =
        generate_embedded_trait_definitions(&input.dir, &input.file, &input.entry);
    let entry_implementation = input.entry.implementation(&input.dir, &input.file);

    let dir_field_factory_definition = generate_factory_trait_definition(&input.dir);
    let file_field_factory_definition = generate_factory_trait_definition(&input.file);
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

fn generate_field_trait_definitions(traits: &[FieldTrait]) -> proc_macro2::TokenStream {
    let mut stream = proc_macro2::TokenStream::new();

    for field in traits {
        stream.extend(field.definition());
    }
    quote! {
        #stream
    }
}

fn generate_embedded_trait_definitions(
    dir_attr: &DirAttr,
    file_attr: &FileAttr,
    entry_attr: &EntryAttr,
) -> proc_macro2::TokenStream {
    let mut stream = proc_macro2::TokenStream::new();

    let traits = dir_attr
        .traits()
        .chain(file_attr.traits())
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
        stream.extend(embedded_trait.definition(&entry_path));
    }
    quote! {
        #stream
    }
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

pub struct FieldTrait {
    field_ident: syn::Ident,
    trait_ident: syn::Ident,
    bound_ident: syn::Ident,
    factory_trait: syn::Ident,
    factory: syn::Path,
    regex: Option<regex::EntryRegex>,
    pattern: Option<pattern::EntryPattern>,

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

pub struct EntryTokens {
    pub struct_path: syn::Path,
    pub kind: EntryKind,
    pub field_name: String,
    pub field_ident: syn::Ident,
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
    /// The absolute fs path for `path` attribute
    pub root: &'a Path,

    /// A nesting level of the entry
    pub level: usize,

    /// User defined field traits
    pub traits: &'a [FieldTrait],

    /// Should we use extensions in idents
    pub with_extensions: bool,

    /// An information about the file system entry
    pub entry: Entry,

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

    /// Information about the `Dir` trait
    pub dir_attr: &'a DirAttr,

    /// Information about the `File` trait
    pub file_attr: &'a FileAttr,

    /// Information about the `Entry` struct
    pub entry_attr: &'a EntryAttr,

    /// The path of the `Entry` struct (including `super::super::...::$ident`)
    pub entry_path: syn::Path,
}

impl<'a> GenerateContext<'a> {
    /// Creates the root-level context
    fn root(
        main_struct_ident: &Ident,
        root: &'a Path,
        traits: &'a [FieldTrait],
        with_extensions: bool,
        dir_attr: &'a DirAttr,
        file_attr: &'a FileAttr,
        entry_attr: &'a EntryAttr,
    ) -> Result<Self, syn::Error> {
        let entry = Entry::root(root).map_err(|e| {
            Error::new_spanned(
                main_struct_ident,
                format!("Unable to read directory '{root:?}' information: {e:?}"),
            )
        })?;

        let mod_name = main_struct_ident.to_string().to_case(Case::Snake);
        let mod_ident = Ident::new(&mod_name, Span::call_site());

        let entry_path = Self::make_nested_path(0, entry_attr.ident().into_owned());
        Ok(Self {
            root,
            level: 0,
            traits,
            with_extensions,
            entry,
            names: UniqueNames::default(),
            struct_name: String::new(),
            struct_ident: main_struct_ident.clone(),
            mod_name,
            mod_ident,
            dir_attr,
            file_attr,
            entry_attr,
            entry_path,
        })
    }

    /// Creates a context for a child of the current
    fn child(&self, entry: Entry) -> Self {
        let struct_name = entry.path().ident.to_case(Case::Pascal);
        let struct_ident = Ident::new_raw(&struct_name, Span::call_site());
        let mod_name = entry.path().ident.to_case(Case::Snake);
        let mod_ident = Ident::new_raw(&mod_name, Span::call_site());
        let level = self.level + 1;
        let entry_path = Self::make_nested_path(level, self.entry_attr.ident().into_owned());

        Self {
            root: self.root,
            level,
            traits: self.traits,
            with_extensions: self.with_extensions,
            entry,
            names: UniqueNames::default(),
            struct_name,
            struct_ident,
            mod_name,
            mod_ident,
            dir_attr: self.dir_attr,
            file_attr: self.file_attr,
            entry_attr: self.entry_attr,
            entry_path,
        }
    }

    fn field_traits_implementation(&self) -> proc_macro2::TokenStream {
        self.traits
            .iter()
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
            EntryKind::File => self.build_file(&entries, &index),
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
            i.relative_path = Path::new(index_relative_path)
                .join(i.relative_path)
                .to_str()
                .unwrap()
                .to_owned();
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
            kind: self.entry.kind(),
            field_name: self.mod_name,
            field_ident: self.mod_ident,
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
            self.root,
            self.with_extensions,
            &mut self.names,
        )
        .map_err(BuildDirError::ReadEntries)?;
        let mut modules = proc_macro2::TokenStream::new();
        for entry in children {
            let child = self.child(entry);
            modules.extend(child.build(entries, index));
        }

        let impl_stream = self.dir_attr.implementation_stream(self, entries, index);
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
    ) -> proc_macro2::TokenStream {
        self.file_attr.implementation_stream(self, entries, index)
    }

    pub fn is_trait_implemented_for(
        &self,
        kind: EntryKind,
        expected: &'static impl EmbeddedTrait,
    ) -> bool {
        match kind {
            EntryKind::Dir => self.dir_attr.is_trait_implemented(expected),
            EntryKind::File => self.file_attr.is_trait_implemented(expected),
        }
    }
}

#[derive(Debug)]
pub enum BuildStreamError {
    Dir(#[allow(dead_code)] BuildDirError),
}

#[derive(Debug)]
pub enum BuildDirError {
    ReadEntries(#[allow(dead_code)] ReadEntriesError),
}

#[cfg(test)]
mod tests {
    use std::fs::{create_dir_all, remove_dir_all};

    use crate::{
        fn_name,
        test_helpers::{create_file, derive_input, tests_dir, PrintToStdOut},
    };

    use super::impl_embed;
    use quote::quote;

    #[test]
    fn check_macros_simple() {
        let current_dir = tests_dir().join(fn_name!());
        if current_dir.exists() {
            remove_dir_all(&current_dir).unwrap();
        }
        create_dir_all(&current_dir).unwrap();
        create_file(current_dir.join("file1.txt"), b"hello");

        let subdir1 = current_dir.join("subdir1");
        create_dir_all(&subdir1).unwrap();
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
        if current_dir.exists() {
            remove_dir_all(&current_dir).unwrap();
        }
        create_dir_all(&current_dir).unwrap();
        create_file(current_dir.join("file1.txt"), b"hello");
        create_file(current_dir.join("file2.txt"), b"world");

        let subdir1 = current_dir.join("subdir1");
        create_dir_all(&subdir1).unwrap();
        create_file(subdir1.join("file1.txt"), b"hello");
        create_file(subdir1.join("file2.txt"), b"world");

        let subdir2 = current_dir.join("subdir2");
        create_dir_all(&subdir2).unwrap();
        create_file(subdir2.join("file1.txt"), b"hello");
        create_file(subdir2.join("file2.txt"), b"world");

        let path = current_dir.to_str().unwrap();

        let input = derive_input(quote! {
            #[derive(Embed)]
            #[embed(
                path = #path,
                field(pattern = "*.txt", factory = "Handle", name = "handle"),
            )]
            pub struct Assets;
        });

        impl_embed(input).print_to_std_out();
    }

    #[test]
    fn check_macros_the_same_normalized_names() {
        let current_dir = tests_dir().join(fn_name!());
        if current_dir.exists() {
            remove_dir_all(&current_dir).unwrap();
        }
        create_dir_all(&current_dir).unwrap();
        create_dir_all(current_dir.join("subdir.txt")).unwrap();
        create_dir_all(current_dir.join("subdir_txt")).unwrap();
        create_dir_all(current_dir.join("subdir+txt")).unwrap();
        create_file(current_dir.join("subdir*txt"), b"hello");
        create_file(current_dir.join("subdir?txt"), b"hello");
        create_file(current_dir.join("subdir-txt"), b"hello");

        let path = current_dir.to_str().unwrap();

        let input = derive_input(quote! {
            #[derive(Embed)]
            #[embed(path = #path, with_extensions = true)]
            pub struct Assets;
        });

        impl_embed(input).print_to_std_out();
    }
}
