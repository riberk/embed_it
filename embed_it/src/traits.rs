use crate::{metadata::Metadata, EmbeddedPath};

pub trait EntryPath {
    /// The entry path
    fn path(&self) -> &'static EmbeddedPath;
}

pub trait Content {
    /// The content of the file
    fn content(&self) -> &'static [u8];
}

pub trait Meta {
    fn metadata(&self) -> &'static Metadata;
}
