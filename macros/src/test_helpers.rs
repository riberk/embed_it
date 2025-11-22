use std::{
    fmt::Debug,
    fs::{self},
    io::Write,
    path::{Path, PathBuf},
    sync::OnceLock,
    thread,
    time::Duration,
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
            remove_and_create_dir_all(&path);
        }

        if !path.is_dir() {
            panic!("'{:?}' dir must be a dir", &path);
        }
        wait_exists(&path);

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
    wait_exists(path);
}

pub fn remove_and_create_dir_all(path: impl AsRef<Path>) {
    let path = path.as_ref();
    if path.exists() {
        remove_dir_all(path);
    }
    create_dir_all(path);
}

pub fn create_dir_all(path: impl AsRef<Path>) {
    let path = path.as_ref();
    std::fs::create_dir_all(path)
        .unwrap_or_else(|e| panic!("Unable to create dir '{path:?}': {e:#?}"));
    wait_exists(path);
}

pub fn remove_dir_all(path: impl AsRef<Path>) {
    let path = path.as_ref();
    std::fs::remove_dir_all(path)
        .unwrap_or_else(|e| panic!("Unable to remove dir '{path:?}': {e:#?}"));
    wait_does_not_exist(path);
}

fn wait_exists(path: &Path) {
    // CI fs is really slow and may fail after creation
    while !path.exists() {
        thread::sleep(Duration::from_millis(50));
    }

    while !std::fs::read_dir(path.parent().expect("no parent"))
        .expect("unable to read parent")
        .flatten()
        .any(|e| e.path() == path)
    {
        thread::sleep(Duration::from_millis(50));
    }
}
fn wait_does_not_exist(path: &Path) {
    // CI fs is really slow and may fail after creation
    while path.exists() {
        thread::sleep(Duration::from_millis(50));
    }

    while std::fs::read_dir(path.parent().expect("no parent"))
        .expect("unable to read parent")
        .flatten()
        .any(|e| e.path() == path)
    {
        thread::sleep(Duration::from_millis(50));
    }
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
