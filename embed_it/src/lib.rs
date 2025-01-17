#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
mod embedded_path;
mod metadata;
mod traits;

pub use embed_it_macros::Embed;
pub use embedded_path::EmbeddedPath;
pub use metadata::Metadata;
pub use traits::{Content, EntryPath, Meta};

#[cfg(feature = "md5")]
pub use traits::Md5Hash;

#[cfg(feature = "sha1")]
pub use traits::Sha1Hash;

#[cfg(feature = "sha2")]
pub use traits::{Sha2_224Hash, Sha2_256Hash, Sha2_384Hash, Sha2_512Hash};

#[cfg(feature = "sha3")]
pub use traits::{Sha3_224Hash, Sha3_256Hash, Sha3_384Hash, Sha3_512Hash};

#[cfg(feature = "blake3")]
pub use traits::Blake3_256Hash;
