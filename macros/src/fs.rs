use std::{
    env::VarError,
    fmt::{Debug, Display},
    fs::read_dir,
    io,
    path::{Path, PathBuf},
};

use embed_it_utils::entry::{Entry, EntryKind};
use regex::{Captures, Regex};
use unicode_ident::{is_xid_continue, is_xid_start};

use crate::{
    embed::{attributes::with_extension::WithExtension, bool_like_enum::BoolLikeEnum},
    utils::unique_names::UniqueNames,
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

    pub fn root_entry(path: &Path) -> Result<Entry<FsInfo>, CreateRootEntryError> {
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
        Ok(Entry::Dir(FsInfo { path, metadata }))
    }

    pub fn read(
        path: &Path,
        root: &Path,
        with_extension: WithExtension,
        names: &mut UniqueNames,
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
                let entry_path = EntryPath::normalize(path, root, with_extension, names)
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

#[derive(Debug)]
pub enum ReadEntriesError {
    UnabeToReadDir(io::Error),
    UnabeToReadEntry(io::Error),
    UnableToNormalizeEntryPath(NormalizePathError),
    UnabeToReadMetadata(io::Error),
}

impl Display for ReadEntriesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadEntriesError::UnabeToReadDir(error) => write!(f, "Unable to read dir: {error}"),
            ReadEntriesError::UnabeToReadEntry(error) => write!(f, "Unable to read entry: {error}"),
            ReadEntriesError::UnableToNormalizeEntryPath(error) => {
                write!(f, "Unable to normalize path: {error}")
            }
            ReadEntriesError::UnabeToReadMetadata(error) => {
                write!(f, "Unable to read metadata: {error}")
            }
        }
    }
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
pub struct UnableToStripRootPrefix {
    origin: PathBuf,
    root: PathBuf,
}

impl Display for UnableToStripRootPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let UnableToStripRootPrefix { origin, root } = self;
        write!(f, "{origin:?} is not a child of the root path {root:?}")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum NormalizePathError {
    NotUtf8(PathBuf),
    NoName,
    UnableToStripRootPrefix(UnableToStripRootPrefix),
}

impl Display for NormalizePathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NormalizePathError::NotUtf8(path_buf) => {
                write!(f, "Path {path_buf:?} is not valid utf8")
            }
            NormalizePathError::NoName => write!(f, "Empty path"),
            NormalizePathError::UnableToStripRootPrefix(e) => write!(f, "{e}"),
        }
    }
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

#[derive(Debug)]
pub enum ExpandPathError {
    Env(String, VarError),
    Canonicalize(String, io::Error),
}

impl Display for ExpandPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpandPathError::Env(name, error) => {
                write!(f, "Environment variable '{name}' error '{error}'")
            }
            ExpandPathError::Canonicalize(path, error) => {
                write!(f, "Unable to canonicalize path '{path}': '{error}'")
            }
        }
    }
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
        embed::attributes::with_extension::WithExtension,
        fn_name,
        fs::{NormalizePathError, UnableToStripRootPrefix},
        test_helpers::tests_dir,
        utils::unique_names::UniqueNames,
    };

    use super::{expand_and_canonicalize, EntryPath, ExpandPathError, ReadEntriesError};

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

    #[test]
    fn read_entries_error_display() {
        let str = ReadEntriesError::UnabeToReadDir(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Internal error",
        ))
        .to_string();
        assert_eq!(&str, "Unable to read dir: Internal error");

        let str = ReadEntriesError::UnabeToReadEntry(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Internal error",
        ))
        .to_string();
        assert_eq!(&str, "Unable to read entry: Internal error");

        let str = ReadEntriesError::UnabeToReadMetadata(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Internal error",
        ))
        .to_string();
        assert_eq!(&str, "Unable to read metadata: Internal error");

        let str =
            ReadEntriesError::UnableToNormalizeEntryPath(NormalizePathError::NoName).to_string();
        assert_eq!(&str, "Unable to normalize path: Empty path");
    }

    #[test]
    fn normalize_path_error_display() {
        let str = NormalizePathError::NoName.to_string();
        assert_eq!(&str, "Empty path");

        let str = NormalizePathError::NotUtf8(PathBuf::from("abcd")).to_string();
        assert_eq!(&str, "Path \"abcd\" is not valid utf8");

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
