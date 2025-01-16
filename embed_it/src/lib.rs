#![doc = include_str!("../../README.md")]
mod embedded_path;
mod metadata;
mod traits;

pub use embedded_path::EmbeddedPath;
pub use macros::Embed;
pub use metadata::Metadata;
pub use traits::{Content, EntryPath, Meta};
