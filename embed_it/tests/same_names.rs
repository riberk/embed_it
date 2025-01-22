use embed_it::Embed;

#[derive(Embed)]
#[embed(
    path = "$CARGO_MANIFEST_DIR/../example_dirs/same_names",
    with_extension = true
)]
pub struct SameNames;

mod tests {
    use embed_it::{Entries, EntryPath};

    use crate::SameNames;

    #[test]
    fn entries() {
        let paths = SameNames
            .entries()
            .iter()
            .map(|e| {
                e.map(|p| p.path(), |p| p.path())
                    .value()
                    .relative_path_str()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            paths,
            ["same+txt", "same.txt", "same_txt", "same)txt", "same-txt", "same=txt"]
        );
    }

    #[test]
    fn fields() {
        assert_eq!(SameNames.same_txt().path().relative_path_str(), "same+txt");
        assert_eq!(
            SameNames.same_txt_1().path().relative_path_str(),
            "same.txt"
        );
        assert_eq!(
            SameNames.same_txt_2().path().relative_path_str(),
            "same_txt"
        );
        assert_eq!(
            SameNames.same_txt_3().path().relative_path_str(),
            "same)txt"
        );
        assert_eq!(
            SameNames.same_txt_4().path().relative_path_str(),
            "same-txt"
        );
        assert_eq!(
            SameNames.same_txt_5().path().relative_path_str(),
            "same=txt"
        );
    }
}
