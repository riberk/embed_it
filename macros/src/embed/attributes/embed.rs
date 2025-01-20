use std::fmt::Display;
use std::path::PathBuf;

use crate::embedded_traits::TraitAttr;
use crate::fs::{expand_and_canonicalize, get_env, Entry, EntryKind, ExpandPathError};

use super::dir::{DirAttr, DirTrait, ParseDirAttrError};
use super::entry::{EntryAttr, EntryStruct};
use super::file::{FileAttr, FileTrait, ParseFileAttrError};
use super::support_alt_separator::SupportAltSeparator;
use super::with_extension::WithExtension;
use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use syn::Ident;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(embed), supports(struct_unit))]
pub struct EmbedInput {
    pub ident: syn::Ident,

    pub path: String,

    #[darling(default)]
    pub with_extension: WithExtension,

    /// If true, before `get` all `\\` characters
    /// will be replaced by `/`. Default: `false`
    #[darling(default)]
    pub support_alt_separator: SupportAltSeparator,

    #[darling(default)]
    pub dir: DirAttr,

    #[darling(default)]
    pub file: FileAttr,

    #[darling(default)]
    pub entry: EntryAttr,
}

#[derive(Debug)]
pub struct GenerationSettings {
    pub main_struct_ident: syn::Ident,

    /// The absolute fs path for `path` attribute
    pub root: PathBuf,

    /// Should we use extensions in idents
    pub with_extension: WithExtension,

    /// If true, before `get` all `\\` characters
    /// will be replaced by `/`
    pub support_alt_separator: SupportAltSeparator,

    /// Information about the `Dir` trait
    pub dir: DirTrait,

    /// Information about the `File` trait
    pub file: FileTrait,

    /// Information about the `Entry` struct
    pub entry: EntryStruct,
}

#[derive(Debug)]
pub enum GenerateFirldTraitsDefinitionError {
    DuplicateTraitWithDifferentMethod(SameTraitName),
}

impl Display for GenerateFirldTraitsDefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenerateFirldTraitsDefinitionError::DuplicateTraitWithDifferentMethod(e) => {
                write!(f, "{e}")
            }
        }
    }
}

#[derive(Debug)]
pub struct SameTraitName {
    trait_name: Ident,
    file_field_name: Ident,
    dir_field_name: Ident,
}

impl Display for SameTraitName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "The same trait name '{}' for 'dir' '{}' field and 'file' {} field",
            self.trait_name, self.dir_field_name, self.file_field_name
        )
    }
}

impl GenerationSettings {
    pub fn trait_for(&self, kind: EntryKind) -> Entry<&DirTrait, &FileTrait> {
        match kind {
            EntryKind::Dir => Entry::Dir(&self.dir),
            EntryKind::File => Entry::File(&self.file),
        }
    }

    pub fn field_traits_definition(
        &self,
    ) -> Result<proc_macro2::TokenStream, GenerateFirldTraitsDefinitionError> {
        let mut result = TokenStream::new();

        for dir_field in self.dir.fields().iter() {
            if let Some(file_field) = self.file.fields().get(dir_field.trait_ident()) {
                return Err(
                    GenerateFirldTraitsDefinitionError::DuplicateTraitWithDifferentMethod(
                        SameTraitName {
                            trait_name: dir_field.trait_ident().clone(),
                            file_field_name: file_field.field_ident().clone(),
                            dir_field_name: dir_field.field_ident().clone(),
                        },
                    ),
                );
            }
            result.extend(dir_field.definition(&self.dir));
        }

        for file_field in self.file.fields().iter() {
            result.extend(file_field.definition(&self.file));
        }

        Ok(result)
    }
}

#[derive(Debug)]
pub enum ParseEmbedInputError {
    ExpandPath(ExpandPathError),
    ParseDir(ParseDirAttrError),
    ParseFile(ParseFileAttrError),
}

impl Display for ParseEmbedInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseEmbedInputError::ExpandPath(e) => write!(f, "Unable to expand a path: {e}"),
            ParseEmbedInputError::ParseDir(e) => write!(f, "Unable to parse `dir` attribute: {e}"),
            ParseEmbedInputError::ParseFile(e) => {
                write!(f, "Unable to parse `file` attribute: {e}")
            }
        }
    }
}

impl TryFrom<EmbedInput> for GenerationSettings {
    type Error = ParseEmbedInputError;

    fn try_from(value: EmbedInput) -> Result<Self, Self::Error> {
        let root = expand_and_canonicalize(&value.path, get_env)
            .map_err(ParseEmbedInputError::ExpandPath)?;
        let dir = DirTrait::try_from(value.dir).map_err(ParseEmbedInputError::ParseDir)?;
        let file = FileTrait::try_from(value.file).map_err(ParseEmbedInputError::ParseFile)?;
        let entry = EntryStruct::from(value.entry);

        Ok(Self {
            main_struct_ident: value.ident,
            root,
            with_extension: value.with_extension,
            support_alt_separator: value.support_alt_separator,
            dir,
            file,
            entry,
        })
    }
}
