#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
mod embedded_path;
mod metadata;
mod traits;

pub use embed_it_macros::Embed;
pub use embedded_path::EmbeddedPath;
pub use metadata::Metadata;
pub use traits::{Content, EntryPath, Meta};
