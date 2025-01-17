use super::dir::DirAttr;
use super::entry::EntryAttr;
use super::field::FieldAttr;
use super::file::FileAttr;
use super::support_alt_separator::SupportAltSeparator;
use super::with_extension::WithExtension;
use darling::FromDeriveInput;

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

    #[darling(default, multiple, rename = "field")]
    pub fields: Vec<FieldAttr>,

    #[darling(default)]
    pub dir: DirAttr,

    #[darling(default)]
    pub file: FileAttr,

    #[darling(default)]
    pub entry: EntryAttr,
}
