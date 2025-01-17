use std::{
    env::VarError,
    fs::read_dir,
    io,
    path::{Path, PathBuf, StripPrefixError},
};

use darling::FromMeta;
use regex::{Captures, Regex};
use syn::parse_quote;
use unicode_ident::{is_xid_continue, is_xid_start};

use crate::{
    embed::{attributes::with_extension::WithExtension, bool_like_enum::BoolLikeEnum},
    unique_names::UniqueNames,
};

const REPLACEMENT_IDENT_CHAR: char = '_';

#[derive(Debug)]
pub enum Entry {
    Dir(EntryInfo),
    File(EntryInfo),
}

#[derive(Debug)]
pub struct EntryInfo {
    path: EntryPath,
    metadata: std::fs::Metadata,
}

impl EntryInfo {
    pub fn path(&self) -> &EntryPath {
        &self.path
    }

    pub fn metadata(&self) -> &std::fs::Metadata {
        &self.metadata
    }
}

impl Entry {
    pub fn path(&self) -> &EntryPath {
        self.info().path()
    }

    pub fn metadata(&self) -> &std::fs::Metadata {
        self.info().metadata()
    }

    pub fn info(&self) -> &EntryInfo {
        match self {
            Entry::Dir(i) => i,
            Entry::File(i) => i,
        }
    }

    pub fn kind(&self) -> EntryKind {
        match self {
            Entry::Dir(_) => EntryKind::Dir,
            Entry::File(_) => EntryKind::File,
        }
    }

    pub fn root(path: &Path) -> Result<Entry, CreateRootEntryError> {
        let metadata = path
            .metadata()
            .map_err(CreateRootEntryError::UnabeToReadMetadata)?;
        let path = EntryPath {
            origin: path
                .to_str()
                .ok_or(CreateRootEntryError::NotUtf8)?
                .to_owned(),
            relative: String::new(),
            ident: String::new(),
            file_name: String::new(),
            file_stem: String::new(),
        };
        Ok(Self::Dir(EntryInfo { path, metadata }))
    }

    pub fn read(
        path: &Path,
        root: &Path,
        with_extension: WithExtension,
        names: &mut UniqueNames,
    ) -> Result<Vec<Entry>, ReadEntriesError> {
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
                let entry_path = EntryPath::normalize(path, root, with_extension, names)
                    .map_err(ReadEntriesError::UnableToNormalizeEntryPath)?;
                Ok(kind.entry(entry_path, metadata))
            })
            .collect::<Result<Vec<_>, ReadEntriesError>>()
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, FromMeta)]
#[darling(rename_all = "lowercase")]
pub enum EntryKind {
    Dir,
    #[default]
    File,
}

impl PartialOrd for EntryKind {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EntryKind {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (EntryKind::Dir, EntryKind::File) => std::cmp::Ordering::Less,
            (EntryKind::File, EntryKind::Dir) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        }
    }
}

impl EntryKind {
    pub fn ident(&self) -> syn::Ident {
        match self {
            EntryKind::File => parse_quote!(File),
            EntryKind::Dir => parse_quote!(Dir),
        }
    }

    pub fn entry(&self, path: EntryPath, metadata: std::fs::Metadata) -> Entry {
        let info = EntryInfo { path, metadata };
        match self {
            EntryKind::Dir => Entry::Dir(info),
            EntryKind::File => Entry::File(info),
        }
    }
}

#[derive(Debug)]
pub enum CreateRootEntryError {
    NotUtf8,
    UnabeToReadMetadata(#[allow(dead_code)] io::Error),
}

#[derive(Debug)]
pub enum ReadEntriesError {
    UnabeToReadDir(#[allow(dead_code)] io::Error),
    UnabeToReadEntry(#[allow(dead_code)] io::Error),
    UnableToNormalizeEntryPath(#[allow(dead_code)] NormalizePathError),
    UnabeToReadMetadata(#[allow(dead_code)] io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntryPath {
    pub origin: String,
    pub relative: String,
    pub ident: String,
    pub file_name: String,
    pub file_stem: String,
}

impl EntryPath {
    pub fn normalize(
        origin: PathBuf,
        root: &Path,
        with_extension: WithExtension,
        names: &mut UniqueNames,
    ) -> Result<EntryPath, NormalizePathError> {
        let origin_str = origin
            .to_str()
            .ok_or_else(|| NormalizePathError::NotUtf8(origin.clone()))?;
        let relative_path = origin.strip_prefix(root).map_err(|e| {
            NormalizePathError::UnableToStripRootPrefix(origin.clone(), root.to_path_buf(), e)
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

        let mut ident = String::with_capacity(ident_candidate.len());

        let mut chars = ident_candidate.chars();
        let start_char = chars.next().ok_or(NormalizePathError::NoName)?;
        if is_xid_start(start_char) {
            ident.push(start_char);
        } else {
            ident.push(REPLACEMENT_IDENT_CHAR);
        }

        for ch in chars {
            if is_xid_continue(ch) {
                ident.push(ch);
            } else {
                ident.push(REPLACEMENT_IDENT_CHAR);
            }
        }

        let ident = names.next(&ident).into_owned();

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
}

#[derive(Debug, PartialEq, Eq)]
pub enum NormalizePathError {
    NotUtf8(PathBuf),
    NoName,
    UnableToStripRootPrefix(PathBuf, PathBuf, StripPrefixError),
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

    std::fs::canonicalize(&path).map_err(|e| ExpandPathError::Fs(path.clone(), e))
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

#[derive(Debug)]
pub enum ExpandPathError {
    Env(#[allow(dead_code)] String, #[allow(dead_code)] VarError),
    Fs(#[allow(dead_code)] String, #[allow(dead_code)] io::Error),
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

    use crate::{
        embed::attributes::with_extension::WithExtension, fn_name, fs::NormalizePathError,
        test_helpers::tests_dir, unique_names::UniqueNames,
    };

    use super::{expand_and_canonicalize, EntryPath, ExpandPathError};

    fn entry_path<P: AsRef<Path>>(origin: P, relative: &str, ident: &str) -> EntryPath {
        EntryPath {
            origin: origin.as_ref().to_str().unwrap().to_owned(),
            relative: relative.to_owned(),
            ident: ident.to_owned(),
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
    fn normalize_path_without_ext() {
        let with_ext = WithExtension::No;
        let mut names = UniqueNames::default();

        let root = Path::new("/home/anonymous");
        let origin = root.join("file1.txt");
        let expected = entry_path(&origin, "file1.txt", "file1");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut names).unwrap(),
            expected
        );

        let origin = root.join("98file1.txt");
        let expected = entry_path(&origin, "98file1.txt", "_8file1");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut names).unwrap(),
            expected
        );

        let origin = root.join("тестテストທົດສອບტესტი.txt");
        let expected = entry_path(&origin, "тестテストທົດສອບტესტი.txt", "тестテストທົດສອບტესტი");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut names).unwrap(),
            expected
        );

        let origin = root.join("t.e.s.t.txt");
        let expected = entry_path(&origin, "t.e.s.t.txt", "t_e_s_t");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut names).unwrap(),
            expected
        );

        let origin = root.join("TeSt.txt");
        let expected = entry_path(&origin, "TeSt.txt", "TeSt");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut names).unwrap(),
            expected
        );

        let origin = root.join("no_ext");
        let expected = entry_path(&origin, "no_ext", "no_ext");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut names).unwrap(),
            expected
        );
    }

    #[test]
    fn normalize_path_with_ext() {
        let with_ext = WithExtension::Yes;
        let root = Path::new("/home/anonymous");
        let mut names = UniqueNames::default();

        let origin = root.join("file1.txt");
        let expected = entry_path(&origin, "file1.txt", "file1_txt");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut names).unwrap(),
            expected
        );

        let origin = root.join("98file1.txt");
        let expected = entry_path(&origin, "98file1.txt", "_8file1_txt");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut names).unwrap(),
            expected
        );

        let origin = root.join("тестテストທົດສອບტესტი.txt");
        let expected = entry_path(
            &origin,
            "тестテストທົດສອບტესტი.txt",
            "тестテストທົດສອບტესტი_txt",
        );
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut names).unwrap(),
            expected
        );

        let origin = root.join("t.e.s.t.txt");
        let expected = entry_path(&origin, "t.e.s.t.txt", "t_e_s_t_txt");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut names).unwrap(),
            expected
        );

        let origin = root.join("TeSt.txt");
        let expected = entry_path(&origin, "TeSt.txt", "TeSt_txt");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut names).unwrap(),
            expected
        );

        let origin = root.join("no_ext");
        let expected = entry_path(&origin, "no_ext", "no_ext");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut names).unwrap(),
            expected
        );
    }

    #[test]
    fn normalize_subpath() {
        let root = Path::new("/home/anonymous");
        let mut names = UniqueNames::default();

        let origin = root.join("a/b/file1.txt");
        let expected = EntryPath {
            origin: origin.to_str().unwrap().to_owned(),
            relative: "a/b/file1.txt".to_owned(),
            ident: "file1_txt".to_owned(),
            file_name: "file1.txt".to_owned(),
            file_stem: "file1".to_owned(),
        };
        assert_eq!(
            EntryPath::normalize(origin, root, WithExtension::Yes, &mut names).unwrap(),
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
                &mut UniqueNames::default()
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
                &mut UniqueNames::default()
            )
            .unwrap_err(),
            NormalizePathError::NotUtf8(origin.clone())
        );
        assert_eq!(
            EntryPath::normalize(
                origin.clone(),
                Path::new(""),
                WithExtension::No,
                &mut UniqueNames::default()
            )
            .unwrap_err(),
            NormalizePathError::NotUtf8(origin.clone())
        );
    }

    #[test]
    fn normalize_non_unique_names() {
        let mut names = UniqueNames::default();
        let root = Path::new("/home/anonymous");
        let with_ext = WithExtension::Yes;

        let origin = root.join("file1.txt");
        let expected = entry_path(&origin, "file1.txt", "file1_txt");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut names).unwrap(),
            expected
        );

        let origin = root.join("file1_txt");
        let expected = entry_path(&origin, "file1_txt", "file1_txt_1");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut names).unwrap(),
            expected
        );

        let origin = root.join("file1+txt");
        let expected = entry_path(&origin, "file1+txt", "file1_txt_2");
        assert_eq!(
            EntryPath::normalize(origin, root, with_ext, &mut names).unwrap(),
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
                    panic!("unknown variable: '{}'", var)
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
            e => panic!("Unexpected error: {:?}", e),
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
            e => panic!("Unexpected error: {:?}", e),
        }
    }
}
