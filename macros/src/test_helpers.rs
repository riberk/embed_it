use std::{
    fmt::Debug,
    fs::{self},
    io::Write,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use syn::DeriveInput;

pub fn derive_input(input: proc_macro2::TokenStream) -> DeriveInput {
    syn::parse2(input).unwrap()
}

pub trait Print {
    fn print_to_std_out(&self);
}

impl<E: Debug> Print for Result<proc_macro2::TokenStream, E> {
    fn print_to_std_out(&self) {
        match self {
            Ok(s) => {
                let s = s.to_string();
                let parsed = syn::parse_file(&s);
                match parsed {
                    Ok(file) => println!("OK: {}", prettyplease::unparse(&file)),
                    Err(e) => panic!("Unable to parse: {e:?}\n\n{s}"),
                }
            }
            Err(e) => {
                panic!("internal error: {e:#?}")
            }
        }
    }
}

pub fn target_dir() -> &'static Path {
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

pub fn tests_dir() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let path = target_dir().join("test_data");
        if !path.exists() {
            create_dir_all(&path);
        }

        if !path.is_dir() {
            panic!("'{:?}' dir must be a dir", &path);
        }
        path
    })
    .as_path()
}

pub fn create_file(path: impl AsRef<Path>, content: &[u8]) {
    let path = path.as_ref();
    fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .unwrap_or_else(|e| panic!("Unable to open file '{path:?}': {e:#?}"))
        .write_all(content)
        .unwrap_or_else(|e| panic!("Unable to write a content into '{path:?}': {e:#?}"));
}

pub fn create_dir_all(path: impl AsRef<Path>) {
    let path = path.as_ref();
    std::fs::create_dir_all(path)
        .unwrap_or_else(|e| panic!("Unable to create dir '{path:?}': {e:#?}"));
}

pub fn remove_dir_all(path: impl AsRef<Path>) {
    let path = path.as_ref();
    std::fs::remove_dir_all(path)
        .unwrap_or_else(|e| panic!("Unable to remove dir '{path:?}': {e:#?}"));
}

#[macro_export]
macro_rules! fn_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name.trim_end_matches("::f")
            .trim_end_matches("::{{closure}}")
            .replace(":", "_")
    }};
}
