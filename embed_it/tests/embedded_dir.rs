use std::path::Path;

use embed_it::{Embed, EmbeddedDir, EmbeddedEntry, EmbeddedFile};

#[derive(Embed)]
#[embed(path = "$CARGO_MANIFEST_DIR/assets")]
pub struct Assets;

mod tests {
    use super::*;
    
    #[test]
    fn fields() {
        assert_eq!(Assets::instance().hello().content(), b"hello");
        assert_eq!(Assets::instance().hello().path(), Path::new("hello.txt"));

        assert_eq!(Assets::instance().one().content(), b"one");
        assert_eq!(Assets::instance().one().path(), Path::new("one.txt"));

        assert_eq!(Assets::instance().world().content(), b"world");
        assert_eq!(Assets::instance().world().path(), Path::new("world.txt"));

        assert_eq!(Assets::instance().one_txt().path(), Path::new("one_txt"));

        assert_eq!(Assets::instance().one_txt().hello().content(), b"hello");
        assert_eq!(
            Assets::instance().one_txt().hello().path(),
            Path::new("hello")
        );

        assert_eq!(Assets::instance().one_txt().world().content(), b"world");
        assert_eq!(
            Assets::instance().one_txt().world().path(),
            Path::new("world")
        );
    }

    #[test]
    fn iter() {
        let entries = Assets::instance().entries().into_iter().collect::<Vec<_>>();
        assert_eq!(entries.len(), 4);

        assert_eq!(
            entries[1]
                .as_file()
                .expect("entries[1] is not a file")
                .content(),
            b"hello"
        );
        assert_eq!(
            entries[2]
                .as_file()
                .expect("entries[2] is not a file")
                .content(),
            b"one"
        );
        assert_eq!(
            entries[3]
                .as_file()
                .expect("entries[3] is not a file")
                .content(),
            b"world"
        );

        let dir_entries = entries[0]
            .as_dir()
            .expect("entries[0] is not a dir")
            .entries()
            .into_iter()
            .collect::<Vec<_>>();
        assert_eq!(dir_entries.len(), 2);

        assert_eq!(
            dir_entries[0]
                .as_file()
                .expect("dir_entries[0] is not a file")
                .content(),
            b"hello"
        );
        assert_eq!(
            dir_entries[1]
                .as_file()
                .expect("dir_entries[1] is not a file")
                .content(),
            b"world"
        );
    }

    #[test]
    fn get() {
        assert!(Assets::instance().get("".as_ref()).is_none());
        assert!(Assets::instance().get("hello".as_ref()).is_none());

        fn get_entry<T>(
            dir: &'static dyn EmbeddedDir<'static>,
            path: &str,
            f: impl FnOnce(EmbeddedEntry<'static, 'static>) -> Option<T>,
            expected: &str,
        ) -> T {
            let entry = dir
                .get(path.as_ref())
                .unwrap_or_else(|| panic!("Unable to find '{}'", path));
            f(entry).unwrap_or_else(|| panic!("'{}' is not a {}", path, expected))
        }

        fn get_file(
            dir: &'static dyn EmbeddedDir<'static>,
            path: &str,
        ) -> &'static EmbeddedFile<'static> {
            get_entry(dir, path, |e| e.as_file(), "file")
        }

        fn get_dir(
            dir: &'static dyn EmbeddedDir<'static>,
            path: &str,
        ) -> &'static dyn EmbeddedDir<'static> {
            get_entry(dir, path, |e| e.as_dir(), "directory")
        }

        assert_eq!(
            get_file(Assets::instance(), "hello.txt").content(),
            b"hello"
        );

        assert_eq!(get_file(Assets::instance(), "one.txt").content(), b"one");

        assert_eq!(
            get_file(Assets::instance(), "world.txt").content(),
            b"world"
        );

        let one_txt = get_dir(Assets::instance(), "one_txt");
        assert_eq!(one_txt.path(), Path::new("one_txt"));
        assert_eq!(get_file(one_txt, "hello").content(), b"hello");
        assert_eq!(get_file(one_txt, "world").content(), b"world");

        assert_eq!(
            get_file(Assets::instance(), "one_txt/hello").content(),
            b"hello"
        );

        assert_eq!(
            get_file(Assets::instance(), "one_txt/world").content(),
            b"world"
        );
    }
}
