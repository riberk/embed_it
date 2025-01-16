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
