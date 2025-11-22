#[derive(embed_it::Embed)]
#[embed(
    path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
    file(derive(StrContent))
)]
pub struct Assets;

mod tests {
    use embed_it::{EmbeddedPath, Entry};

    use crate::{Dir, DynDir, DynFile, EntryDir, EntryFile};

    use super::Assets;

    fn get_entry<D: Dir + ?Sized, T>(
        dir: &D,
        path: &str,
        f: impl FnOnce(&Entry<DynDir, DynFile>) -> Option<T>,
        expected: &str,
    ) -> T {
        let entry = dir
            .get(path)
            .unwrap_or_else(|| panic!("Unable to find '{path}'"));
        f(entry).unwrap_or_else(|| panic!("'{path}' is not a {expected}"))
    }

    fn get_file<D: Dir + ?Sized>(dir: &D, path: &str) -> &'static dyn EntryFile {
        get_entry(dir, path, |e| e.file().map(|f| f.into_file()), "file")
    }

    fn get_dir<D: Dir + ?Sized>(dir: &D, path: &str) -> &'static dyn EntryDir {
        get_entry(dir, path, |e| e.dir().map(|d| d.into_dir()), "directory")
    }

    #[test]
    fn fields() {
        assert_eq!(Assets.hello().content(), b"hello");
        assert_eq!(Assets.hello().str_content(), "hello");
        assert_eq!(
            Assets.hello().path(),
            &EmbeddedPath::new("hello.txt", "hello.txt", "hello")
        );

        assert_eq!(Assets.one().content(), b"one");
        assert_eq!(
            Assets.one().path(),
            &EmbeddedPath::new("one.txt", "one.txt", "one")
        );

        assert_eq!(Assets.world().content(), b"world");
        assert_eq!(
            Assets.world().path(),
            &EmbeddedPath::new("world.txt", "world.txt", "world")
        );

        assert_eq!(
            Assets.one_txt().path(),
            &EmbeddedPath::new("one_txt", "one_txt", "one_txt")
        );

        assert_eq!(Assets.one_txt().hello().content(), b"hello");
        assert_eq!(
            Assets.one_txt().hello().path(),
            &EmbeddedPath::new("one_txt/hello", "hello", "hello")
        );

        assert_eq!(Assets.one_txt().world().content(), b"world");
        assert_eq!(
            Assets.one_txt().world().path(),
            &EmbeddedPath::new("one_txt/world", "world", "world")
        );
    }

    #[test]
    fn iter() {
        let entries = Assets.entries();
        assert_eq!(entries.len(), 4);

        let entry_0 = entries[0].dir().expect("not a dir");
        assert_eq!(entry_0.path().relative_path_str(), "one_txt");
        assert_eq!(
            entries[1]
                .file()
                .expect("not a file")
                .path()
                .relative_path_str(),
            "hello.txt"
        );
        assert_eq!(
            entries[2]
                .file()
                .expect("not a file")
                .path()
                .relative_path_str(),
            "one.txt"
        );
        assert_eq!(
            entries[3]
                .file()
                .expect("not a file")
                .path()
                .relative_path_str(),
            "world.txt"
        );

        let sub_entries = entry_0.entries();
        assert_eq!(sub_entries.len(), 2);
        assert_eq!(
            sub_entries[0]
                .file()
                .expect("not a file")
                .path()
                .relative_path_str(),
            "one_txt/hello"
        );
        assert_eq!(
            sub_entries[1]
                .file()
                .expect("not a file")
                .path()
                .relative_path_str(),
            "one_txt/world"
        );
    }

    #[test]
    fn get() {
        assert!(Assets.get("hello").is_none());

        assert_eq!(get_dir(&Assets, "").path().relative_path_str(), "");
        assert_eq!(get_file(&Assets, "hello.txt").content(), b"hello");
        assert_eq!(get_file(&Assets, "one.txt").content(), b"one");
        assert_eq!(get_file(&Assets, "world.txt").content(), b"world");
        assert_eq!(get_file(&Assets, "one_txt/hello").content(), b"hello");
        assert_eq!(get_file(&Assets, "one_txt/world").content(), b"world");

        let one_txt = get_dir(&Assets, "one_txt");
        assert_eq!(one_txt.path().relative_path_str(), "one_txt");
        assert_eq!(get_dir(one_txt, "").path().relative_path_str(), "one_txt");
        assert_eq!(get_file(one_txt, "hello").content(), b"hello");
        assert_eq!(
            get_file(one_txt, "hello").path().relative_path_str(),
            "one_txt/hello"
        );
        assert_eq!(get_file(one_txt, "world").content(), b"world");
        assert_eq!(
            get_file(one_txt, "world").path().relative_path_str(),
            "one_txt/world"
        );
    }

    #[test]
    fn direct_child_count() {
        assert_eq!(4, Assets.direct_child_count());
        assert_eq!(2, Assets.one_txt().direct_child_count());
    }

    #[test]
    fn recursive_child_count() {
        assert_eq!(6, Assets.recursive_child_count());
        assert_eq!(2, Assets.one_txt().recursive_child_count());
    }
}
