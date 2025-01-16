use std::{
    fs::{create_dir_all, remove_file, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

fn main() {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("Unable to get env CARGO_MANIFEST_DIR");
    let project_path = PathBuf::from(&manifest_dir);

    generate_md_of_dir_structure(
        &project_path.join("assets"),
        &project_path.join("../doc/assets_structure.md"),
    );
    generate_md_of_dir_structure(
        &project_path.join("same_names"),
        &project_path.join("../doc/same_names_structure.md"),
    );
}

fn generate_md_of_dir_structure(path: &Path, write_into: &Path) {
    if write_into.is_file() {
        remove_file(write_into)
            .unwrap_or_else(|e| panic!("Unable to remove old version of '{write_into:?}': {e:?}"));
    }

    if write_into.is_dir() {
        panic!("'{write_into:?}' must be a file path, but it is a directory");
    }

    create_dir_all(write_into.parent().unwrap()).unwrap_or_else(|e| {
        panic!("Unable to create a parent directory of a '{write_into:?}': {e:?}")
    });

    let file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(write_into)
        .unwrap_or_else(|e| panic!("Unable to create a file '{write_into:?}': {e:?}"));
    let mut buf = BufWriter::new(file);
    build_markdown_tree(&mut buf, path, 0);
    buf.flush()
        .unwrap_or_else(|e| panic!("Unable to flush content into '{write_into:?}': {e:?}"));
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum EntryPath {
    Dir(PathBuf),
    File(PathBuf),
}

impl EntryPath {
    pub fn path(&self) -> &Path {
        match self {
            EntryPath::Dir(path) => path,
            EntryPath::File(path) => path,
        }
    }
}

fn build_markdown_tree(buf: &mut impl std::io::Write, path: &Path, indent: usize) {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(path).expect("Unable to read dir") {
        let entry = entry.expect("Unable to read entry");
        let entry = match entry.file_type() {
            Ok(t) => {
                if t.is_dir() {
                    EntryPath::Dir(entry.path())
                } else if t.is_file() {
                    EntryPath::File(entry.path())
                } else {
                    continue;
                }
            }
            Err(_) => continue,
        };
        entries.push(entry);
    }
    entries.sort();

    let prefix = "  ".repeat(indent);

    for entry_path in entries {
        let name = entry_path
            .path()
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        match &entry_path {
            EntryPath::Dir(p) => {
                writeln!(buf, "{prefix}- {name}/").expect("Unable to write");
                build_markdown_tree(buf, p, indent + 1);
            }
            EntryPath::File(_) => {
                writeln!(buf, "{prefix}- {name}").expect("Unable to write");
            }
        }
    }
}
