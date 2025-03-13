#![allow(clippy::needless_doctest_main)]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
mod embedded_path;
mod metadata;
mod traits;

pub use embed_it_macros::Embed;
pub use embedded_path::EmbeddedPath;
pub use metadata::Metadata;
pub use traits::{
    ChildOf, Content, DirectChildCount, Entries, EntryPath, Index, Meta, RecursiveChildCount,
    StrContent,
};

pub use embed_it_utils::entry::Entry;

#[cfg(feature = "md5")]
pub use traits::hashes::Md5Hash;

#[cfg(feature = "sha1")]
pub use traits::hashes::Sha1Hash;

#[cfg(feature = "sha2")]
pub use traits::hashes::{Sha2_224Hash, Sha2_256Hash, Sha2_384Hash, Sha2_512Hash};

#[cfg(feature = "sha3")]
pub use traits::hashes::{Sha3_224Hash, Sha3_256Hash, Sha3_384Hash, Sha3_512Hash};

#[cfg(feature = "blake3")]
pub use traits::hashes::Blake3_256Hash;

#[cfg(feature = "gzip")]
pub use traits::compression::GzipContent;

#[cfg(feature = "brotli")]
pub use traits::compression::BrotliContent;

#[cfg(feature = "zstd")]
pub use traits::compression::ZstdContent;
