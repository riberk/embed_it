use std::{
    env,
    fs::{create_dir_all, remove_dir_all, OpenOptions},
    io::Write,
    path::Path,
};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    for feature in env::vars()
        .filter_map(|(name, _)| name.strip_prefix("CARGO_FEATURE_").map(|v| v.to_owned()))
    {
        let cfg = feature_config(&feature);
        let path = Path::new(&out_dir).join(&feature);
        let path_str = path.to_str().unwrap();
        let path_var = feature.as_str();
        create_dir(&path);
        create_tree(
            &path,
            cfg.depth,
            cfg.directories_per_level,
            cfg.files_per_directory,
        );
        println!("cargo::rustc-env={path_var}={path_str}")
    }
}

fn feature_config(feature: &str) -> Config {
    match feature {
        "BENCH_DIRS" => Config {
            depth: 0,
            directories_per_level: 1000,
            files_per_directory: 0,
        },
        "BENCH_FILES" => Config {
            depth: 0,
            directories_per_level: 0,
            files_per_directory: 1000,
        },
        "BENCH_NESTED_DIRS" => Config {
            depth: 100,
            directories_per_level: 1,
            files_per_directory: 0,
        },
        "BENCH_NESTING" => Config {
            depth: 4,
            directories_per_level: 3,
            files_per_directory: 15,
        },
        "BENCH_LOTS_OF_FILES" => Config {
            depth: 0,
            directories_per_level: 0,
            files_per_directory: 15_000,
        },
        "BENCH_LOTS_OF_DIRS" => Config {
            depth: 0,
            directories_per_level: 15_000,
            files_per_directory: 0,
        },
        "BENCH_LOTS_OF_NESTING_ITEMS" => Config {
            depth: 3,
            directories_per_level: 10,
            files_per_directory: 5,
        },
        
        _ => {
            panic!("Unknonw feature '{feature}'")
        }
    }
}

struct Config {
    depth: i32,
    directories_per_level: usize,
    files_per_directory: usize,
}

fn create_tree(path: &Path, depth: i32, directories_per_level: usize, files_per_directory: usize) {
    if depth < 0 {
        return;
    }

    for i in 0..files_per_directory {
        create_file(&path.join(format!("file_{i}.txt")));
    }

    for i in 0..directories_per_level {
        let path = path.join(format!("dir_{i}"));
        create_dir(&path);
        create_tree(&path, depth - 1, directories_per_level, files_per_directory);
    }
}

fn create_dir(path: &Path) {
    if path.exists() {
        remove_dir_all(path).unwrap_or_else(|e| panic!("Unable to remove dir {path:?}: {e:?}"));
    }
    create_dir_all(path).unwrap_or_else(|e| panic!("Unable to create dir {path:?}: {e:?}"));
}

fn create_file(path: &Path) {
    const BUF: &[u8] = &[b'a'; 128];

    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .unwrap_or_else(|e| panic!("Unable to create file {path:?}: {e:?}"));
    file.write_all(BUF).unwrap();
}
