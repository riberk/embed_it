use embed_it::Embed;

#[derive(Embed)]
#[embed(
    path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
    with_extension = true
)]
pub struct Assets;

mod tests {
    use embed_it::{Content, EmbeddedPath, EntryPath};

    use super::*;

    fn get_entry<D: Dir + ?Sized, T>(
        dir: &D,
        path: &str,
        f: impl FnOnce(&Entry) -> Option<T>,
        expected: &str,
    ) -> T {
        let entry = dir
            .get(path.as_ref())
            .unwrap_or_else(|| panic!("Unable to find '{}'", path));
        f(entry).unwrap_or_else(|| panic!("'{}' is not a {}", path, expected))
    }

    fn get_file<D: Dir + ?Sized>(dir: &D, path: &str) -> &'static dyn File {
        get_entry(dir, path, |e| e.file(), "file")
    }

    fn get_dir<D: Dir + ?Sized>(dir: &D, path: &str) -> &'static dyn Dir {
        get_entry(dir, path, |e| e.dir(), "directory")
    }

    #[test]
    fn fields() {
        assert_eq!(Assets.hello_txt().content(), b"hello");
        assert_eq!(
            Assets.hello_txt().path(),
            &EmbeddedPath::new("hello.txt", "hello.txt", "hello")
        );

        assert_eq!(Assets.one_txt_1().content(), b"one");
        assert_eq!(
            Assets.one_txt_1().path(),
            &EmbeddedPath::new("one.txt", "one.txt", "one")
        );

        assert_eq!(Assets.world_txt().content(), b"world");
        assert_eq!(
            Assets.world_txt().path(),
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
        assert!(Assets.get("".as_ref()).is_none());
        assert!(Assets.get("hello".as_ref()).is_none());

        assert_eq!(get_file(&Assets, "hello.txt").content(), b"hello");
        assert_eq!(get_file(&Assets, "one.txt").content(), b"one");
        assert_eq!(get_file(&Assets, "world.txt").content(), b"world");
        assert_eq!(get_file(&Assets, "one_txt/hello").content(), b"hello");
        assert_eq!(get_file(&Assets, "one_txt/world").content(), b"world");

        let one_txt = get_dir(&Assets, "one_txt");
        assert_eq!(one_txt.path().relative_path_str(), "one_txt");
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
}
