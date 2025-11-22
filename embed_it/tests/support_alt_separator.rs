mod alt_sep {
    #[derive(embed_it::Embed)]
    #[embed(
        path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
        support_alt_separator = "yes"
    )]
    pub struct Assets;
}

mod no_alt_sep {
    #[derive(embed_it::Embed)]
    #[embed(
        path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
        support_alt_separator = "no"
    )]
    pub struct Assets;
}

mod tests {
    use super::*;

    #[test]
    fn get() {
        assert_eq!(
            alt_sep::Assets
                .get("one_txt/hello")
                .unwrap()
                .file()
                .unwrap()
                .content(),
            b"hello"
        );
        assert_eq!(
            alt_sep::Assets
                .get("one_txt\\hello")
                .unwrap()
                .file()
                .unwrap()
                .content(),
            b"hello"
        );

        assert!(no_alt_sep::Assets.get("one_txt\\hello").is_none());
        assert_eq!(
            no_alt_sep::Assets
                .get("one_txt/hello")
                .unwrap()
                .file()
                .unwrap()
                .content(),
            b"hello"
        );
    }
}
