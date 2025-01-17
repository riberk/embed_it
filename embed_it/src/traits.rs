use crate::{metadata::Metadata, EmbeddedPath};

/// Provides full information about a path of an entry
pub trait EntryPath {
    /// The entry path
    fn path(&self) -> &'static EmbeddedPath;
}

/// Provides the file content
pub trait Content {
    /// Get the content of the file
    fn content(&self) -> &'static [u8];
}

/// Provides metadata of an entry   
pub trait Meta {
    /// Get the metadata of the entry
    fn metadata(&self) -> &'static Metadata;
}

#[cfg(feature = "md5")]
pub trait Md5Hash {
    fn md5(&self) -> &'static [u8; 16];
}

#[cfg(feature = "sha1")]
pub trait Sha1Hash {
    fn sha1(&self) -> &'static [u8; 20];
}

#[cfg(feature = "sha2")]
pub trait Sha2_224Hash {
    fn sha2_224(&self) -> &'static [u8; 28];
}

#[cfg(feature = "sha2")]
pub trait Sha2_256Hash {
    fn sha2_256(&self) -> &'static [u8; 32];
}

#[cfg(feature = "sha2")]
pub trait Sha2_384Hash {
    fn sha2_384(&self) -> &'static [u8; 48];
}

#[cfg(feature = "sha2")]
pub trait Sha2_512Hash {
    fn sha2_512(&self) -> &'static [u8; 64];
}

#[cfg(feature = "sha3")]
pub trait Sha3_224Hash {
    fn sha3_224(&self) -> &'static [u8; 28];
}

#[cfg(feature = "sha3")]
pub trait Sha3_256Hash {
    fn sha3_256(&self) -> &'static [u8; 32];
}

#[cfg(feature = "sha3")]
pub trait Sha3_384Hash {
    fn sha3_384(&self) -> &'static [u8; 48];
}

#[cfg(feature = "sha3")]
pub trait Sha3_512Hash {
    fn sha3_512(&self) -> &'static [u8; 64];
}

#[cfg(feature = "blake3")]
pub trait Blake3_256Hash {
    fn blake3_256(&self) -> &'static [u8; 32];
}
