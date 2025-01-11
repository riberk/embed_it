mod embedded_dir;
mod embedded_entry;
mod embedded_file;

pub use embedded_dir::{get_from_dir, DirEntries, EmbeddedDir, Entries, Instance};
pub use embedded_entry::EmbeddedEntry;
pub use embedded_file::EmbeddedFile;
pub use macros::Assets;