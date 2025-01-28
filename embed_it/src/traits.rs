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

/// A trait for accessing the number of direct children in a dir-like structure.
///
/// This trait provides a method to retrieve a constant value representing the number of
/// direct children
pub trait DirectChildCount {
    /// Returns the number of direct children in the current directory.
    ///
    /// If an implementation is generated by the macro, this value is constant and determined at compile time
    fn direct_child_count(&self) -> usize;
}

/// A trait for accessing the total number of children, including nested subdirectories.
///
/// This trait provides a method to retrieve a value representing the total number
/// of children, including all nested subdirectories
pub trait RecursiveChildCount {
    /// Returns the total number of children, including nested subdirectories and their contents.
    ///
    /// If an implementation is generated by the macro, this value is constant and determined at compile time
    fn recursive_child_count(&self) -> usize;
}

/// It's a marker trait, that indicates the current type is a child of `T` on the `LEVEL` level.
pub trait ChildOf<T, const LEVEL: usize> {}

#[cfg(feature = "any-hash")]
pub mod hashes;
