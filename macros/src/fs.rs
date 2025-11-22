use std::{
    borrow::Cow,
    env::VarError,
    fmt::Debug,
    fs::read_dir,
    io,
    path::{Path, PathBuf},
};

use convert_case::{Case, Casing};
use embed_it_utils::entry::{Entry, EntryKind};
use proc_macro2::Span;
use quote::ToTokens;
use regex::{Captures, Regex};
use syn::Ident;
use unicode_ident::{is_xid_continue, is_xid_start};

use crate::{
    embed::{attributes::with_extension::WithExtension, bool_like_enum::BoolLikeEnum},
    utils::unique_names::UniqueIdents,
};

const REPLACEMENT_IDENT_CHAR: char = '_';

#[derive(Debug)]
pub struct FsInfo {
    path: EntryPath,
    metadata: std::fs::Metadata,
}

impl FsInfo {
    pub fn path(&self) -> &EntryPath {
        &self.path
    }

    pub fn metadata(&self) -> &std::fs::Metadata {
        &self.metadata
    }

    pub fn root_entry(
        path: &Path,
        main_ident: Ident,
    ) -> Result<Entry<FsInfo>, CreateRootEntryError> {
        let metadata = path
            .metadata()
            .map_err(CreateRootEntryError::UnabeToReadMetadata)?;
        let path = EntryPath {
            origin: path
                .to_str()
                .ok_or(CreateRootEntryError::NotUtf8)?
                .to_owned(),
            relative: String::new(),
            ident: EntryIdent::root(main_ident),
            file_name: String::new(),
            file_stem: String::new(),
        };
        Ok(Entry::Dir(FsInfo { path, metadata }))
    }

    pub fn read(
        path: &Path,
        root: &Path,
        with_extension: WithExtension,
        idents: &mut UniqueIdents,
    ) -> Result<Vec<Entry<FsInfo>>, ReadEntriesError> {
        let dir = read_dir(path).map_err(ReadEntriesError::UnabeToReadDir)?;

        let mut entries = Vec::new();

        for entry in dir {
            let entry = entry.map_err(ReadEntriesError::UnabeToReadEntry)?;
            let path = entry.path();
            if path.is_dir() {
                entries.push((EntryKind::Dir, path));
            } else if path.is_file() {
                entries.push((EntryKind::File, path));
            } else {
                continue;
            };
        }

        entries.sort();

        entries
            .into_iter()
            .map(|(kind, path)| {
                let metadata = path
                    .metadata()
                    .map_err(ReadEntriesError::UnabeToReadMetadata)?;
                let entry_path = EntryPath::normalize(path, root, with_extension, idents)
                    .map_err(ReadEntriesError::UnableToNormalizeEntryPath)?;
                Ok(Self::from_kind(kind, entry_path, metadata))
            })
            .collect::<Result<Vec<_>, ReadEntriesError>>()
    }

    pub fn from_kind(
        kind: EntryKind,
        path: EntryPath,
        metadata: std::fs::Metadata,
    ) -> Entry<FsInfo> {
        let info = Self { path, metadata };
        match kind {
            EntryKind::Dir => Entry::Dir(info),
            EntryKind::File => Entry::File(info),
        }
    }
}

#[derive(Debug)]
pub enum CreateRootEntryError {
    NotUtf8,
    UnabeToReadMetadata(io::Error),
}

#[derive(Debug, derive_more::Display)]
pub enum ReadEntriesError {
    #[display("unable to read a dir: {_0}")]
    UnabeToReadDir(io::Error),

    #[display("unable to read an entry: {_0}")]
    UnabeToReadEntry(io::Error),

    #[display("unable to normalize a path: {_0}")]
    UnableToNormalizeEntryPath(NormalizePathError),

    #[display("unable to read a metadata: {_0}")]
    UnabeToReadMetadata(io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryPath {
    pub origin: String,
    pub relative: String,
    pub ident: EntryIdent,
    pub file_name: String,
    pub file_stem: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryIdent {
    /// Module like identifier: `snake_case`
    module_like: StrIdent,

    /// Struct like identifier: `PascalCase`
    struct_like: StrIdent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrIdent {
    name: String,
    ident: Ident,
}

impl ToTokens for StrIdent {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.ident().to_tokens(tokens);
    }
}

impl StrIdent {
    pub fn new<'a, T: Into<Cow<'a, str>>>(name: T) -> Self {
        let name = name.into().into_owned();
        Self {
            ident: Ident::new(&name, Span::call_site()),
            name,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }
}

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub struct EmptryString;

impl EntryIdent {
    pub fn root(ident: Ident) -> Self {
        let string_ident = ident.to_string();
        let mod_name = string_ident.to_case(Case::Snake);
        let mod_ident = Ident::new(&mod_name, Span::call_site());
        let struct_name = string_ident;
        let struct_ident = ident;
        Self {
            module_like: StrIdent {
                name: mod_name,
                ident: mod_ident,
            },
            struct_like: StrIdent {
                name: struct_name,
                ident: struct_ident,
            },
        }
    }

    fn from_key(
        prefixed: bool,
        ident_key: &str,
        case: Case,
        names: impl FnOnce(&str) -> Option<usize>,
    ) -> StrIdent {
        let candidate = {
            let delim = case.delim();
            let mut candidate = ident_key;
            if delim.is_empty() {
                candidate
            } else {
                while let Some(s) = candidate.strip_prefix(delim) {
                    candidate = s;
                }

                while let Some(s) = candidate.strip_suffix(delim) {
                    candidate = s;
                }

                candidate
            }
        };

        let mut candidate = candidate.to_case(case);
        if prefixed {
            candidate.insert(0, '_');
        }

        if candidate.is_empty() {
            candidate.push('_');
        }

        if let Some(unique_postfix) = names(&candidate) {
            use std::fmt::Write;
            write!(&mut candidate, "{}{unique_postfix}", case.delim()).unwrap();
        };
        StrIdent::new(candidate)
    }

    fn create(str: &str, idents: &mut UniqueIdents) -> Result<EntryIdent, EmptryString> {
        let mut ident_key = String::with_capacity(str.len() + 1);

        let mut chars = str.chars().peekable();
        let start_char = *chars.peek().ok_or(EmptryString)?;
        let prefixed = !(start_char == '_' || is_xid_start(start_char));

        for ch in chars {
            if is_xid_continue(ch) {
                ident_key.push(ch);
            } else {
                ident_key.push(REPLACEMENT_IDENT_CHAR);
            }
        }

        Ok(Self {
            module_like: Self::from_key(prefixed, &ident_key, Case::Snake, |s| {
                idents.next_module(s)
            }),
            struct_like: Self::from_key(prefixed, &ident_key, Case::Pascal, |s| {
                idents.next_struct(s)
            }),
        })
    }

    pub fn module_like(&self) -> &StrIdent {
        &self.module_like
    }

    pub fn struct_like(&self) -> &StrIdent {
        &self.struct_like
    }
}

impl EntryPath {
    pub fn normalize(
        origin: PathBuf,
        root: &Path,
        with_extension: WithExtension,
        idents: &mut UniqueIdents,
    ) -> Result<EntryPath, NormalizePathError> {
        let origin_str = origin
            .to_str()
            .ok_or_else(|| NormalizePathError::NotUtf8(origin.clone()))?;
        let relative_path = origin.strip_prefix(root).map_err(|_| {
            NormalizePathError::UnableToStripRootPrefix(UnableToStripRootPrefix {
                origin: origin.clone(),
                root: root.to_path_buf(),
            })
        })?;

        // it's guaranteed, that relative_path, file_name and file_stem are utf8, because origin is utf8
        let relative = relative_path.to_str().unwrap();

        let file_name = relative_path
            .file_name()
            .ok_or(NormalizePathError::NoName)?
            .to_str()
            .unwrap();
        let file_stem = relative_path
            .file_stem()
            .ok_or(NormalizePathError::NoName)?
            .to_str()
            .unwrap();

        let ident_candidate = if with_extension.as_bool() {
            file_name
        } else {
            file_stem
        };

        let ident =
            EntryIdent::create(ident_candidate, idents).map_err(|_| NormalizePathError::NoName)?;

        let relative = if cfg!(target_os = "windows") {
            relative.replace('\\', "/")
        } else {
            relative.to_owned()
        };

        Ok(EntryPath {
            relative,
            ident,
            file_name: file_name.to_owned(),
            file_stem: file_stem.to_owned(),
            origin: origin_str.to_owned(),
        })
    }

    pub fn relative_path(&self) -> &Path {
        Path::new(&self.relative)
    }

    pub fn origin_path(&self) -> &Path {
        Path::new(&self.origin)
    }

    pub fn ident(&self) -> &EntryIdent {
        &self.ident
    }
}

#[derive(Debug, PartialEq, Eq, derive_more::Display)]
#[display("{:?} is not a child of the root path {:?}", self.origin, self.root)]
pub struct UnableToStripRootPrefix {
    origin: PathBuf,
    root: PathBuf,
}

#[derive(Debug, PartialEq, Eq, derive_more::Display)]
pub enum NormalizePathError {
    #[display("path {_0:?} is not a valid utf8 string")]
    NotUtf8(PathBuf),

    #[display("empty path")]
    NoName,

    UnableToStripRootPrefix(UnableToStripRootPrefix),
}

pub fn expand_and_canonicalize(
    input: &str,
    get_env: impl Fn(&str) -> Result<String, VarError>,
) -> Result<PathBuf, ExpandPathError> {
    let re = Regex::new(r"\$(\w+)|\$\{([^}]+)\}").unwrap();

    let replacement = |caps: &Captures| -> Result<String, ExpandPathError> {
        let var = caps
            .get(1)
            .or_else(|| caps.get(2))
            .map(|m| m.as_str())
            .expect("Unable to find variable");
        let value = get_env(var).map_err(|e| ExpandPathError::Env(var.to_owned(), e))?;
        Ok(value)
    };
    let path = replace_all(&re, input, replacement)?;

    let path = if std::path::MAIN_SEPARATOR == '\\' {
        path.replace("/", std::path::MAIN_SEPARATOR_STR)
    } else {
        path.replace("\\", std::path::MAIN_SEPARATOR_STR)
    };

    std::fs::canonicalize(&path).map_err(|e| ExpandPathError::Canonicalize(path.clone(), e))
}

pub fn replace_all<E>(
    re: &Regex,
    haystack: &str,
    replacement: impl Fn(&Captures) -> Result<String, E>,
) -> Result<String, E> {
    let mut new = String::with_capacity(haystack.len());
    let mut last_match = 0;
    for caps in re.captures_iter(haystack) {
        let m = caps.get(0).unwrap();
        new.push_str(&haystack[last_match..m.start()]);
        new.push_str(&replacement(&caps)?);
        last_match = m.end();
    }
    new.push_str(&haystack[last_match..]);
    Ok(new)
}

#[derive(Debug, derive_more::Display)]
pub enum ExpandPathError {
    #[display("environment variable '{_0}' error: '{_1}'")]
    Env(String, VarError),

    #[display("unable to canonicalize path '{_0}': '{_1}'")]
    Canonicalize(String, io::Error),
}

pub fn get_env(variable: &str) -> Result<String, VarError> {
    std::env::var(variable)
}

#[cfg(test)]
mod tests {
    use std::{
        env::VarError,
        fs::{create_dir_all, remove_dir_all},
        path::{Path, PathBuf},
    };

    use syn::parse_quote;

    use crate::{
        embed::attributes::with_extension::WithExtension,
        fn_name,
        fs::{EntryIdent, NormalizePathError, StrIdent, UnableToStripRootPrefix},
        test_helpers::tests_dir,
        utils::unique_names::UniqueIdents,
    };

    use pretty_assertions::assert_eq;

    use super::{EntryPath, ExpandPathError, ReadEntriesError, expand_and_canonicalize};

    fn entry_path<P: AsRef<Path>>(
        origin: P,
        relative: &str,
        mod_ident: &str,
        struct_ident: &str,
    ) -> EntryPath {
        EntryPath {
            origin: origin.as_ref().to_str().unwrap().to_owned(),
            relative: relative.to_owned(),
            ident: EntryIdent {
                module_like: StrIdent::new(mod_ident),
                struct_like: StrIdent::new(struct_ident),
            },
            file_name: Path::new(relative)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned(),
            file_stem: Path::new(relative)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned(),
        }
    }

    #[test]
    fn new_str_ident() {
        assert_eq!(
            StrIdent::new("AaA".to_owned()),
            StrIdent {
                name: "AaA".to_owned(),
                ident: parse_quote!(AaA)
            }
        );
    }

    #[test]
    fn create_entry_ident_unique_name() {
        let ident = EntryIdent::create("AAaaAA", &mut UniqueIdents::default()).unwrap();
        assert_eq!(
            ident,
            EntryIdent {
                module_like: StrIdent::new("a_aaa_aa"),
                struct_like: StrIdent::new("AAaaAa")
            }
        );

        let ident = EntryIdent::create("aa_aa_aa", &mut UniqueIdents::default()).unwrap();
        assert_eq!(
            ident,
            EntryIdent {
                module_like: StrIdent::new("aa_aa_aa"),
                struct_like: StrIdent::new("AaAaAa")
            }
        );

        let ident = EntryIdent::create("aa_AA_aa", &mut UniqueIdents::default()).unwrap();
        assert_eq!(
            ident,
            EntryIdent {
                module_like: StrIdent::new("aa_aa_aa"),
                struct_like: StrIdent::new("AaAaAa")
            }
        );

        let ident = EntryIdent::create("___AA___", &mut UniqueIdents::default()).unwrap();
        assert_eq!(
            ident,
            EntryIdent {
                module_like: StrIdent::new("aa"),
                struct_like: StrIdent::new("Aa")
            }
        );

        let ident = EntryIdent::create("123", &mut UniqueIdents::default()).unwrap();
        assert_eq!(
            ident,
            EntryIdent {
                module_like: StrIdent::new("_123"),
                struct_like: StrIdent::new("_123"),
            }
        );

        let ident = EntryIdent::create("_", &mut UniqueIdents::default()).unwrap();
        assert_eq!(
            ident,
            EntryIdent {
                module_like: StrIdent::new("_"),
                struct_like: StrIdent::new("_"),
            }
        );

        let ident = EntryIdent::create("-", &mut UniqueIdents::default()).unwrap();
        assert_eq!(
            ident,
            EntryIdent {
                module_like: StrIdent::new("_"),
                struct_like: StrIdent::new("_"),
            }
        );
    }

    #[test]
    fn normalize_path_without_ext() {
        let with_ext = WithExtension::No;
        let mut idents = UniqueIdents::default();

        let root = Path::new("/home/anonymous");

        let origin = root.join("1.txt");
        let expected = entry_path(&origin, "1.txt", "_1", "_1");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut idents).unwrap(),
            expected
        );

        let origin = root.join("file1.txt");
        let expected = entry_path(&origin, "file1.txt", "file_1", "File1");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut idents).unwrap(),
            expected
        );

        let origin = root.join("98file1.txt");
        let expected = entry_path(&origin, "98file1.txt", "_98_file_1", "_98File1");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut idents).unwrap(),
            expected
        );

        let origin = root.join("тестテストທົດສອບტესტი.txt");
        let expected = entry_path(
            &origin,
            "тестテストທົດສອບტესტი.txt",
            "тестテストທົດສອບტესტი",
            "Тестテストທົດສອບტესტი",
        );
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut idents).unwrap(),
            expected
        );

        let origin = root.join("t.e.s.t.txt");
        let expected = entry_path(&origin, "t.e.s.t.txt", "t_e_s_t", "TEST");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut idents).unwrap(),
            expected
        );

        let origin = root.join("TeSt.txt");
        let expected = entry_path(&origin, "TeSt.txt", "te_st", "TeSt");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut idents).unwrap(),
            expected
        );

        let origin = root.join("no_ext");
        let expected = entry_path(&origin, "no_ext", "no_ext", "NoExt");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut idents).unwrap(),
            expected
        );
    }

    #[test]
    fn normalize_path_with_ext() {
        let with_ext = WithExtension::Yes;
        let root = Path::new("/home/anonymous");
        let mut idents = UniqueIdents::default();

        let origin = root.join("1.txt");
        let expected = entry_path(&origin, "1.txt", "_1_txt", "_1Txt");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut idents).unwrap(),
            expected
        );

        let origin = root.join("file1.txt");
        let expected = entry_path(&origin, "file1.txt", "file_1_txt", "File1Txt");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut idents).unwrap(),
            expected
        );

        let origin = root.join("98file1.txt");
        let expected = entry_path(&origin, "98file1.txt", "_98_file_1_txt", "_98File1Txt");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut idents).unwrap(),
            expected
        );

        let origin = root.join("тестテストທົດສອບტესტი.txt");
        let expected = entry_path(
            &origin,
            "тестテストທົດສອບტესტი.txt",
            "тестテストທົດສອບტესტი_txt",
            "ТестテストທົດສອບტესტიTxt",
        );
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut idents).unwrap(),
            expected
        );

        let origin = root.join("t.e.s.t.txt");
        let expected = entry_path(&origin, "t.e.s.t.txt", "t_e_s_t_txt", "TESTTxt");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut idents).unwrap(),
            expected
        );

        let origin = root.join("TeSt.txt");
        let expected = entry_path(&origin, "TeSt.txt", "te_st_txt", "TeStTxt");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut idents).unwrap(),
            expected
        );

        let origin = root.join("no_ext");
        let expected = entry_path(&origin, "no_ext", "no_ext", "NoExt");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut idents).unwrap(),
            expected
        );
    }

    #[test]
    fn normalize_subpath() {
        let root = Path::new("/home/anonymous");
        let mut idents = UniqueIdents::default();

        let origin = root.join("a/b/file1.txt");
        let expected = EntryPath {
            origin: origin.to_str().unwrap().to_owned(),
            relative: "a/b/file1.txt".to_owned(),
            ident: EntryIdent {
                module_like: StrIdent::new("file_1_txt".to_owned()),
                struct_like: StrIdent::new("File1Txt".to_owned()),
            },
            file_name: "file1.txt".to_owned(),
            file_stem: "file1".to_owned(),
        };
        assert_eq!(
            EntryPath::normalize(origin, root, WithExtension::Yes, &mut idents,).unwrap(),
            expected
        );
    }

    #[test]
    fn normalize_path_empty_path() {
        assert_eq!(
            EntryPath::normalize(
                PathBuf::from(""),
                Path::new(""),
                WithExtension::Yes,
                &mut UniqueIdents::default(),
            )
            .unwrap_err(),
            NormalizePathError::NoName
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn normalize_path_not_utf8() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};

        let origin = PathBuf::from(OsString::from_vec(vec![255]));
        assert_eq!(
            EntryPath::normalize(
                origin.clone(),
                Path::new(""),
                WithExtension::Yes,
                &mut UniqueIdents::default(),
            )
            .unwrap_err(),
            NormalizePathError::NotUtf8(origin.clone())
        );
        assert_eq!(
            EntryPath::normalize(
                origin.clone(),
                Path::new(""),
                WithExtension::No,
                &mut UniqueIdents::default(),
            )
            .unwrap_err(),
            NormalizePathError::NotUtf8(origin.clone())
        );
    }

    #[test]
    fn normalize_non_unique_names() {
        let mut idents = UniqueIdents::default();

        let root = Path::new("/home/anonymous");
        let with_ext = WithExtension::Yes;

        let origin = root.join("file1.txt");
        let expected = entry_path(&origin, "file1.txt", "file_1_txt", "File1Txt");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut idents).unwrap(),
            expected
        );

        let origin = root.join("file1_txt");
        let expected = entry_path(&origin, "file1_txt", "file_1_txt_1", "File1Txt1");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut idents).unwrap(),
            expected
        );

        let origin = root.join("file1+txt");
        let expected = entry_path(&origin, "file1+txt", "file_1_txt_2", "File1Txt2");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut idents).unwrap(),
            expected
        );
    }

    #[test]
    fn expand_and_canonicalize_pass() {
        let dir_name = fn_name!();
        let current_dir = tests_dir().join(&dir_name);
        if current_dir.exists() {
            remove_dir_all(&current_dir).unwrap();
        }
        create_dir_all(&current_dir).unwrap();

        let res = expand_and_canonicalize(
            &format!(
                "../target/test_data/{}/../{}/../$DIR/../${{DIR}}",
                &dir_name, &dir_name
            ),
            |var| {
                if var == "DIR" {
                    Ok(dir_name.clone())
                } else {
                    panic!("unknown variable: '{var}'")
                }
            },
        )
        .unwrap_or_else(|e| panic!("Unable to canonicalize '{e:#?}'"));

        assert_eq!(
            res,
            std::fs::canonicalize(&current_dir)
                .unwrap_or_else(|e| panic!("Unable to canonicalize '{current_dir:?}': {e:#?}"))
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn expand_and_canonicalize_not_utf() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};

        let expected_err = VarError::NotUnicode(OsString::from_vec(vec![1]));

        let err = expand_and_canonicalize("./$Q", |_| Err(expected_err.clone())).unwrap_err();

        match err {
            ExpandPathError::Env(var, var_error) => {
                assert_eq!("Q", &var);
                assert_eq!(expected_err, var_error);
            }
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn expand_and_canonicalize_not_present() {
        let expected_err = VarError::NotPresent;

        let err = expand_and_canonicalize("./$Q", |_| Err(expected_err.clone())).unwrap_err();

        match err {
            ExpandPathError::Env(var, var_error) => {
                assert_eq!("Q", &var);
                assert_eq!(expected_err, var_error);
            }
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn read_entries_error_display() {
        let str = ReadEntriesError::UnabeToReadDir(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Internal error",
        ))
        .to_string();
        assert_eq!(&str, "unable to read a dir: Internal error");

        let str = ReadEntriesError::UnabeToReadEntry(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Internal error",
        ))
        .to_string();
        assert_eq!(&str, "unable to read an entry: Internal error");

        let str = ReadEntriesError::UnabeToReadMetadata(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Internal error",
        ))
        .to_string();
        assert_eq!(&str, "unable to read a metadata: Internal error");

        let str =
            ReadEntriesError::UnableToNormalizeEntryPath(NormalizePathError::NoName).to_string();
        assert_eq!(&str, "unable to normalize a path: empty path");
    }

    #[test]
    fn normalize_path_error_display() {
        let str = NormalizePathError::NoName.to_string();
        assert_eq!(&str, "empty path");

        let str = NormalizePathError::NotUtf8(PathBuf::from("abcd")).to_string();
        assert_eq!(&str, "path \"abcd\" is not a valid utf8 string");

        let str = NormalizePathError::UnableToStripRootPrefix(UnableToStripRootPrefix {
            origin: PathBuf::from("origin_path"),
            root: PathBuf::from("root_path"),
        })
        .to_string();
        assert_eq!(
            &str,
            "\"origin_path\" is not a child of the root path \"root_path\""
        );
    }
}
