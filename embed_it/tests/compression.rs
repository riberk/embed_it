#[cfg(feature = "gzip")]
pub mod gzip {
    use embed_it::GzipContent;
    use hex_literal::hex;

    #[derive(embed_it::Embed)]
    #[embed(
        path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
        file(derive_default_traits = false, derive(Gzip)),
        dir(derive_default_traits = false)
    )]
    pub struct Assets;

    #[test]
    fn check() {
        assert_eq!(
            Assets.hello().gzip_content(),
            &hex!("1f8b08000000000002ffcb48cdc9c9070086a6103605000000")
        );

        assert_eq!(
            Assets.world().gzip_content(),
            &hex!("1f8b08000000000002ff2bcf2fca4901004311773a05000000")
        );
    }
}

#[cfg(feature = "brotli")]
pub mod brotli {
    use embed_it::BrotliContent;
    use hex_literal::hex;

    #[derive(embed_it::Embed)]
    #[embed(
        path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
        file(derive_default_traits = false, derive(Brotli)),
        dir(derive_default_traits = false)
    )]
    pub struct Assets;

    #[test]
    fn check() {
        assert_eq!(Assets.hello().brotli_content(), &hex!("0b028068656c6c6f03"));
        assert_eq!(Assets.world().brotli_content(), &hex!("0b0280776f726c6403"));
    }
}

#[cfg(feature = "zstd")]
pub mod zstd {
    use embed_it::ZstdContent;
    use hex_literal::hex;

    #[derive(embed_it::Embed)]
    #[embed(
        path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
        file(derive_default_traits = false, derive(Zstd)),
        dir(derive_default_traits = false)
    )]
    pub struct Assets;

    #[test]
    fn check() {
        assert_eq!(
            Assets.hello().zstd_content(),
            &hex!("28b52ffd008829000068656c6c6f")
        );

        assert_eq!(
            Assets.world().zstd_content(),
            &hex!("28b52ffd0088290000776f726c64")
        );
    }
}
