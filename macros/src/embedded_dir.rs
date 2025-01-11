use std::{
    borrow::Cow,
    collections::HashMap,
    env::VarError,
    fs::{self, read_dir},
    io,
    path::{Path, PathBuf},
};

use convert_case::{Case, Casing};
use darling::FromDeriveInput;
use proc_macro2::Span;
use quote::{format_ident, quote};
use regex::{Captures, Regex};
use syn::{DeriveInput, Error, Ident};
use unicode_ident::{is_xid_continue, is_xid_start};

const REPLACEMENT_IDENT_CHAR: char = '_';

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(assets), supports(struct_unit))]
pub struct EmbeddedDirInput {
    path: String,
    with_extensions: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Entry {
    Dir(EntryPath),
    File(EntryPath),
}

impl Entry {
    pub fn path(&self) -> &EntryPath {
        match self {
            Entry::Dir(entry_path) => entry_path,
            Entry::File(entry_path) => entry_path,
        }
    }
}

pub(crate) fn impl_assets(
    input: DeriveInput,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let main_struct_ident = &input.ident;

    let input = EmbeddedDirInput::from_derive_input(&input)?;

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
    let entry_path = EntryPath::normalize(root, with_extensions).map_err(|e| {
        Error::new_spanned(
            main_struct_ident,
            format!("Unable to normalize main path: {:#?}", e),
        )
    })?;

    let mod_name = format!(
        "{}_data",
        main_struct_ident.to_string().to_case(Case::Snake)
    );
    let mod_ident = Ident::new(&mod_name, Span::call_site());
    let struct_ident = Ident::new(&mod_name.to_case(Case::Pascal), Span::call_site());

    let stream =
        generate_struct_and_module(&struct_ident, &entry_path, with_extensions).map_err(|e| {
            Error::new_spanned(
                &struct_ident,
                format!("Unable to generate main struct: {:#?}", e),
            )
        })?;

    let stream = quote! {
        impl #main_struct_ident {
            pub fn instance() -> &'static #mod_ident::#struct_ident<'static> {
                <#mod_ident::#struct_ident as ::include_assets::Instance>::instance()
            }
        }
        pub mod #mod_ident {
            #stream
        }
    };
    Ok(stream)
}

fn make_struct_ident(name: &str) -> Ident {
    Ident::new_raw(&name.to_case(Case::Pascal), Span::call_site())
}

fn generate_struct_and_module(
    struct_ident: &Ident,
    path: &EntryPath,
    with_extensions: bool,
) -> Result<proc_macro2::TokenStream, ReadEntriesError> {
    let entries = read_entries(&path.origin, with_extensions)?;

    let mut names = UniqueNames::default();
    let mut fields = proc_macro2::TokenStream::new();
    let mut methods = proc_macro2::TokenStream::new();
    let mut modules = proc_macro2::TokenStream::new();
    let mut by_index = proc_macro2::TokenStream::new();
    let mut data_fields = proc_macro2::TokenStream::new();
    let mut index_data = proc_macro2::TokenStream::new();
    let mut debug = proc_macro2::TokenStream::new();

    let mut len: usize = 0;
    for (idx, entry) in entries.iter().enumerate() {
        len += 1;
        let path = entry.path();
        let field_name = names.next(&path.ident);
        let field_ident = Ident::new_raw(&field_name, Span::call_site());

        let relative_path = path
            .relative
            .as_path()
            .to_str()
            .ok_or_else(|| ReadEntriesError::NotUtf8(path.relative.clone()))?;
        let absolute_path = path
            .origin
            .as_path()
            .to_str()
            .ok_or_else(|| ReadEntriesError::NotUtf8(path.origin.clone()))?;
        index_data.extend(quote! {
            (::std::path::Path::new(#relative_path), #idx),
        });

        match entry {
            Entry::Dir(_) => {
                let struct_ident = make_struct_ident(&field_name);

                methods.extend(quote! {
                    pub fn #field_ident(&self) -> &#field_ident::#struct_ident<'data> {
                        <#field_ident::#struct_ident as ::include_assets::Instance>::instance()
                    }
                });
                let module_stream =
                    generate_struct_and_module(&struct_ident, path, with_extensions)?;

                modules.extend(quote! {
                    pub mod #field_ident {
                        #module_stream
                    }
                });

                by_index.extend(quote! {
                    #idx => Some(::include_assets::EmbeddedEntry::Dir(self.#field_ident())),
                });

                debug.extend(quote! {
                    d.field(#field_name, self.#field_ident());
                });
            }
            Entry::File(_) => {
                fields.extend(quote! {
                    #field_ident: ::include_assets::EmbeddedFile<'data>,
                });

                methods.extend(quote! {
                    pub fn #field_ident(&self) -> &::include_assets::EmbeddedFile<'data> {
                        &self.#field_ident
                    }
                });
                by_index.extend(quote! {
                    #idx => Some(::include_assets::EmbeddedEntry::File(self.#field_ident())),
                });

                data_fields.extend(quote! {
                    #field_ident: ::include_assets::EmbeddedFile::new(::std::path::Path::new(#relative_path), include_bytes!(#absolute_path)),
                });

                debug.extend(quote! {
                    d.field(#field_name, &self.#field_ident);
                });
            }
        }
    }

    let index_name = names.next("___embedded_files_index");
    let index_field_ident = Ident::new_raw(&index_name, Span::call_site());

    fields.extend(quote! {
        #index_field_ident: ::std::collections::HashMap<&'data ::std::path::Path, usize>,
    });

    let relative_path = path
        .relative
        .as_path()
        .to_str()
        .ok_or_else(|| ReadEntriesError::NotUtf8(path.relative.clone()))?;

    let data_ident = Ident::new_raw(
        &format_ident!("{}_{}", struct_ident, "DATA")
            .to_string()
            .to_case(Case::ScreamingSnake),
        Span::call_site(),
    );

    let struct_name = struct_ident.to_string();
    let index_format = format!("<len: {len}>");
    debug.extend(quote! {
        d.field(#index_name, &#index_format);
    });

    Ok(quote! {
        pub struct #struct_ident<'data> {
            #fields
        }

        impl<'data> #struct_ident<'data> {
            #methods
        }

        static #data_ident: ::std::sync::LazyLock<#struct_ident<'static>> = ::std::sync::LazyLock::new(|| #struct_ident {
            #data_fields
            #index_field_ident: std::collections::HashMap::from([
                #index_data
            ]),
        });

        impl ::include_assets::Instance<'static> for #struct_ident<'static> {
            fn instance() -> &'static Self {
                &#data_ident
            }
        }

        impl<'data> ::include_assets::EmbeddedDir<'data> for #struct_ident<'data> {
            fn path(&self) -> &'data ::std::path::Path {
                ::std::path::Path::new(#relative_path)
            }

            fn entries(&self) -> ::include_assets::DirEntries<'_, 'data> {
                ::include_assets::DirEntries::new(self)
            }


            fn by_index(&self, idx: usize) -> Option<::include_assets::EmbeddedEntry<'_, 'data>> {
                match idx {
                    #by_index
                    _ => None,
                }
            }

            fn len(&self) -> usize {
                #len
            }

            fn index(&self) -> &::std::collections::HashMap<&'data ::std::path::Path, usize> {
                &self.#index_field_ident
            }

            fn get(&self, path: &::std::path::Path) -> Option<::include_assets::EmbeddedEntry<'_, 'data>> {
                ::include_assets::get_from_dir(self, path)
            }
        }

        impl<'data> std::fmt::Debug for #struct_ident<'data> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut d = f.debug_struct(#struct_name);
                #debug
                d.finish()
            }
        }

        #modules
    })
}

#[derive(Default)]
pub struct UniqueNames<'a>(HashMap<&'a str, usize>);

impl<'a> UniqueNames<'a> {
    pub fn next(&mut self, name: &'a str) -> Cow<'a, str> {
        let count_with_name = self.0.entry(name).or_insert(0_usize);
        let res = if *count_with_name > 0 {
            Cow::Owned(format!("{}_{}", name, count_with_name))
        } else {
            Cow::Borrowed(name)
        };
        *count_with_name += 1;
        res
    }
}

fn read_entries(path: &Path, with_extensions: bool) -> Result<Vec<Entry>, ReadEntriesError> {
    let dir = read_dir(path).map_err(ReadEntriesError::UnabeToReadDir)?;

    let mut entries = Vec::new();

    for entry in dir {
        let entry = entry.map_err(ReadEntriesError::UnabeToReadEntry)?;
        let path = entry.path();
        let create = if path.is_dir() {
            Entry::Dir
        } else if path.is_file() {
            Entry::File
        } else {
            continue;
        };

        let entry_path = EntryPath::normalize(path, with_extensions)
            .map_err(ReadEntriesError::UnableToNormalizeEntryPath)?;

        entries.push(create(entry_path));
    }

    entries.sort();
    Ok(entries)
}

#[derive(Debug)]
pub enum ReadEntriesError {
    UnabeToReadDir(#[allow(dead_code)] io::Error),
    UnabeToReadEntry(#[allow(dead_code)] io::Error),
    UnableToNormalizeEntryPath(#[allow(dead_code)] NormalizePathError),
    NotUtf8(#[allow(dead_code)] PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntryPath {
    origin: PathBuf,
    relative: PathBuf,
    ident: String,
}

impl EntryPath {
    #[cfg(test)]
    pub fn new<'a, I: Into<Cow<'a, str>>>(origin: &Path, relative: &Path, ident: I) -> Self {
        Self {
            origin: origin.to_path_buf(),
            relative: relative.to_path_buf(),
            ident: ident.into().into_owned(),
        }
    }

    pub fn normalize(
        origin: PathBuf,
        with_extension: bool,
    ) -> Result<EntryPath, NormalizePathError> {
        let relative = Path::new(
            origin
                .as_path()
                .file_name()
                .ok_or(NormalizePathError::NoName)?,
        );

        let ident_candidate = if with_extension {
            relative.as_os_str()
        } else {
            relative.file_stem().ok_or(NormalizePathError::NoName)?
        }
        .to_str()
        .ok_or_else(|| NormalizePathError::NotUtf8(origin.clone()))?;

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

        Ok(EntryPath {
            relative: relative.to_owned(),
            origin,
            ident,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum NormalizePathError {
    NotUtf8(PathBuf),
    NoName,
}

fn expand_and_canonicalize(
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
    fs::canonicalize(&path).map_err(|e| ExpandPathError::Fs(path.clone(), e))
}

fn replace_all<E>(
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

fn get_env(variable: &str) -> Result<String, VarError> {
    std::env::var(variable)
}

#[cfg(test)]
pub mod tests {
    use std::{
        env::VarError,
        ffi::OsString,
        fs::{self, create_dir, create_dir_all, remove_dir_all},
        io::Write,
        os::unix::ffi::OsStringExt,
        path::{Path, PathBuf},
        sync::OnceLock,
    };

    use crate::embedded_dir::{make_struct_ident, ExpandPathError};

    use super::{
        expand_and_canonicalize, generate_struct_and_module, impl_assets, EntryPath,
        NormalizePathError,
    };
    use pretty_assertions::assert_eq;
    use quote::quote;

    use std::fmt::Debug;

    use syn::DeriveInput;

    pub fn derive_input(input: proc_macro2::TokenStream) -> DeriveInput {
        syn::parse2(input).unwrap()
    }

    pub trait PrintToStdOut {
        fn print_to_std_out(&self);
    }

    impl<E: Debug> PrintToStdOut for Result<proc_macro2::TokenStream, E> {
        fn print_to_std_out(&self) {
            match self {
                Ok(s) => {
                    let s = s.to_string();
                    let parsed = syn::parse_file(&s);
                    match parsed {
                        Ok(file) => println!("{}", prettyplease::unparse(&file)),
                        Err(e) => panic!("Unable to parse: {e:?}\n\n{s}"),
                    }
                }
                Err(e) => {
                    panic!("internal error: {e:#?}")
                }
            }
        }
    }

    fn target_dir() -> &'static Path {
        static DIR: OnceLock<PathBuf> = OnceLock::new();
        DIR.get_or_init(|| {
            let mut dir = None;
            for candidate_parent in Path::new(
                &std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not defined"),
            )
            .ancestors()
            {
                let candidate = candidate_parent.join("target");
                if candidate.is_dir() {
                    dir = Some(candidate);
                    break;
                }
            }

            dir.expect("Unable to find target directory")
        })
        .as_path()
    }

    fn tests_dir() -> &'static Path {
        static DIR: OnceLock<PathBuf> = OnceLock::new();
        DIR.get_or_init(|| {
            let path = target_dir().join("test_data");
            if !path.exists() {
                create_dir(&path)
                    .unwrap_or_else(|e| panic!("Unable to create dir '{path:?}': {e:#?}"));
            }

            if !path.is_dir() {
                panic!("'{:?}' dir must be a dir", &path);
            }
            path
        })
        .as_path()
    }

    #[test]
    fn normalize_path_without_ext() {
        let with_ext = false;

        let origin = Path::new("/home/anonymous/file1.txt");
        let expected = EntryPath::new(origin, "file1.txt".as_ref(), "file1");
        assert_eq!(
            EntryPath::normalize(origin.to_path_buf(), with_ext).unwrap(),
            expected
        );

        let origin = Path::new("/home/anonymous/98file1.txt");
        let expected = EntryPath::new(origin, "98file1.txt".as_ref(), "_8file1");
        assert_eq!(
            EntryPath::normalize(origin.to_path_buf(), with_ext).unwrap(),
            expected
        );

        let origin = Path::new("/home/anonymous/тестテストທົດສອບტესტი.txt");
        let expected = EntryPath::new(
            origin,
            "тестテストທົດສອບტესტი.txt".as_ref(),
            "тестテストທົດສອບტესტი",
        );
        assert_eq!(
            EntryPath::normalize(origin.to_path_buf(), with_ext).unwrap(),
            expected
        );

        let origin = Path::new("/home/anonymous/t.e.s.t.txt");
        let expected = EntryPath::new(origin, "t.e.s.t.txt".as_ref(), "t_e_s_t");
        assert_eq!(
            EntryPath::normalize(origin.to_path_buf(), with_ext).unwrap(),
            expected
        );

        let origin = Path::new("/home/anonymous/TeSt.txt");
        let expected = EntryPath::new(origin, "TeSt.txt".as_ref(), "TeSt");
        assert_eq!(
            EntryPath::normalize(origin.to_path_buf(), with_ext).unwrap(),
            expected
        );

        let origin = Path::new("/home/anonymous/no_ext");
        let expected = EntryPath::new(origin, "no_ext".as_ref(), "no_ext");
        assert_eq!(
            EntryPath::normalize(origin.to_path_buf(), with_ext).unwrap(),
            expected
        );
    }

    #[test]
    fn normalize_path_with_ext() {
        let with_ext = true;

        let origin = Path::new("/home/anonymous/file1.txt");
        let expected = EntryPath::new(origin, "file1.txt".as_ref(), "file1_txt");
        assert_eq!(
            EntryPath::normalize(origin.to_path_buf(), with_ext).unwrap(),
            expected
        );

        let origin = Path::new("/home/anonymous/98file1.txt");
        let expected = EntryPath::new(origin, "98file1.txt".as_ref(), "_8file1_txt");
        assert_eq!(
            EntryPath::normalize(origin.to_path_buf(), with_ext).unwrap(),
            expected
        );

        let origin = Path::new("/home/anonymous/тестテストທົດສອບტესტი.txt");
        let expected = EntryPath::new(
            origin,
            "тестテストທົດສອບტესტი.txt".as_ref(),
            "тестテストທົດສອບტესტი_txt",
        );
        assert_eq!(
            EntryPath::normalize(origin.to_path_buf(), with_ext).unwrap(),
            expected
        );

        let origin = Path::new("/home/anonymous/t.e.s.t.txt");
        let expected = EntryPath::new(origin, "t.e.s.t.txt".as_ref(), "t_e_s_t_txt");
        assert_eq!(
            EntryPath::normalize(origin.to_path_buf(), with_ext).unwrap(),
            expected
        );

        let origin = Path::new("/home/anonymous/TeSt.txt");
        let expected = EntryPath::new(origin, "TeSt.txt".as_ref(), "TeSt_txt");
        assert_eq!(
            EntryPath::normalize(origin.to_path_buf(), with_ext).unwrap(),
            expected
        );

        let origin = Path::new("/home/anonymous/no_ext");
        let expected = EntryPath::new(origin, "no_ext".as_ref(), "no_ext");
        assert_eq!(
            EntryPath::normalize(origin.to_path_buf(), with_ext).unwrap(),
            expected
        );
    }

    #[test]
    fn normalize_path_empty_path() {
        assert_eq!(
            EntryPath::normalize(PathBuf::from(""), true).unwrap_err(),
            NormalizePathError::NoName
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn normalize_path_not_utf8() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};

        let origin = PathBuf::from(OsString::from_vec(vec![255]));
        assert_eq!(
            EntryPath::normalize(origin.clone(), true).unwrap_err(),
            NormalizePathError::NotUtf8(origin.clone())
        );
        assert_eq!(
            EntryPath::normalize(origin.clone(), false).unwrap_err(),
            NormalizePathError::NotUtf8(origin.clone())
        );
    }

    #[test]
    fn expand_and_canonicalize_pass() {
        let dir_name = "expand_and_canonicalize_pass";
        let current_dir = tests_dir().join(dir_name);
        if current_dir.exists() {
            remove_dir_all(&current_dir).unwrap();
        }
        create_dir_all(&current_dir).unwrap();

        let res = expand_and_canonicalize(
            &format!(
                "../target/test_data/{}/../{}/../$DIR/../${{DIR}}",
                dir_name, dir_name
            ),
            |var| {
                if var == "DIR" {
                    Ok(dir_name.to_owned())
                } else {
                    panic!("unknown variable: '{}'", var)
                }
            },
        )
        .unwrap_or_else(|e| panic!("Unable to canonicalize '{e:#?}'"));

        assert_eq!(res, current_dir);
    }

    #[test]
    fn expand_and_canonicalize_not_utf() {
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
    fn check_generate_struct_and_module_simple() {
        let dir_name = "check_generate_struct_and_module_simple";
        let current_dir = tests_dir().join(dir_name);
        if current_dir.exists() {
            remove_dir_all(&current_dir).unwrap();
        }
        create_dir_all(&current_dir).unwrap();
        fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(current_dir.join("file1.txt"))
            .unwrap()
            .write_all(b"hello")
            .unwrap();

        let path = EntryPath::normalize(current_dir, false).unwrap();
        let res = generate_struct_and_module(&make_struct_ident("Assets"), &path, false);

        res.print_to_std_out();
    }

    #[test]
    fn check_macros() {
        let dir_name = "check_macros";
        let current_dir = tests_dir().join(dir_name);
        if current_dir.exists() {
            remove_dir_all(&current_dir).unwrap();
        }
        create_dir_all(&current_dir).unwrap();
        fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(current_dir.join("file1.txt"))
            .unwrap()
            .write_all(b"hello")
            .unwrap();
        fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(current_dir.join("file2.txt"))
            .unwrap()
            .write_all(b"world")
            .unwrap();

        let subdir1 = current_dir.join("subdir1");
        create_dir_all(&subdir1).unwrap();
        fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(subdir1.join("file1.txt"))
            .unwrap()
            .write_all(b"hello")
            .unwrap();
        fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(subdir1.join("file2.txt"))
            .unwrap()
            .write_all(b"world")
            .unwrap();

        let subdir2 = current_dir.join("subdir2");
        create_dir_all(&subdir2).unwrap();
        fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(subdir2.join("file1.txt"))
            .unwrap()
            .write_all(b"hello")
            .unwrap();
        fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(subdir2.join("file2.txt"))
            .unwrap()
            .write_all(b"world")
            .unwrap();

        let path = current_dir.to_str().unwrap();

        let input = derive_input(quote! {
            #[derive(Assets)]
            #[assets(path = #path)]
            pub struct Assets;
        });

        impl_assets(input).print_to_std_out();
    }
}
