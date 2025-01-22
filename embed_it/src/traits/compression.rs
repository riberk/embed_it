#[cfg(feature = "gzip")]
/// Trait providing access to content compressed in Gzip format.
///
/// It's compatible with HTTP `Content-Encoding` `gzip`.
///
/// ```rust
/// #[cfg(feature = "gzip")]
/// pub mod gzip {
///     use ::embed_it::GzipContent;
///     use hex_literal::hex;
///     
///     #[derive(::embed_it::Embed)]
///     #[embed(
///         path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
///         file(derive_default_traits = false, derive(Gzip))
///     )]
///     pub struct Assets;
///     
///     fn main() {
///         assert_eq!(Assets.hello().gzip_content(), &hex!("1f8b08000000000002ffcb48cdc9c9070086a6103605000000"));
///         assert_eq!(Assets.world().gzip_content(), &hex!("1f8b08000000000002ff2bcf2fca4901004311773a05000000"));
///     }
/// }
/// ```
pub trait GzipContent {
    /// Returns the compressed content in Gzip format.
    fn gzip_content(&self) -> &'static [u8];
}

#[cfg(feature = "brotli")]
/// Trait providing access to content compressed in Brotli format.
///
/// It's compatible with HTTP `Content-Encoding` `brotli`.
///
/// ```rust
/// #[cfg(feature = "brotli")]
/// pub mod gzip {
///     use ::embed_it::BrotliContent;
///     use hex_literal::hex;
///     
///     #[derive(::embed_it::Embed)]
///     #[embed(
///         path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
///         file(derive_default_traits = false, derive(Brotli))
///     )]
///     pub struct Assets;
///     
///     fn main() {
///         assert_eq!(Assets.hello().brotli_content(), &hex!("0b028068656c6c6f03"));
///         assert_eq!(Assets.world().brotli_content(), &hex!("0b0280776f726c6403"));
///     }
/// }
/// ```
pub trait BrotliContent {
    /// Returns the compressed content in Brotli format.
    fn brotli_content(&self) -> &'static [u8];
}

#[cfg(feature = "zstd")]
/// Trait providing access to content compressed in Zstd format.
///
/// It's compatible with HTTP `Content-Encoding` `zstd`.
/// ```rust
/// #[cfg(feature = "zstd")]
/// pub mod gzip {
///     use ::embed_it::ZstdContent;
///     use hex_literal::hex;
///     
///     #[derive(::embed_it::Embed)]
///     #[embed(
///         path = "$CARGO_MANIFEST_DIR/../example_dirs/assets",
///         file(derive_default_traits = false, derive(Zstd))
///     )]
///     pub struct Assets;
///     
///     fn main() {
///         assert_eq!(Assets.hello().zstd_content(), &hex!("28b52ffd008829000068656c6c6f"));
///         assert_eq!(Assets.world().zstd_content(), &hex!("28b52ffd0088290000776f726c64"));
///     }
/// }
/// ```
pub trait ZstdContent {
    /// Returns the compressed content in Zstd format.
    fn zstd_content(&self) -> &'static [u8];
}
