use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmbeddedPath {
    relative_path: &'static str,
    name: &'static str,
    stem: &'static str,
}

impl EmbeddedPath {
    pub const fn new(relative_path: &'static str, name: &'static str, stem: &'static str) -> Self {
        Self {
            relative_path,
            name,
            stem,
        }
    }

    /// Relative to the root assets dir path. Is's guaranteed, that valid utf8 inside the path,
    /// because it creates from a &str. Cost-free
    pub fn relative_path(&self) -> &'static Path {
        Path::new(self.relative_path)
    }

    /// [`self.relative_path`] as a &str. Cost-free
    pub fn relative_path_str(&self) -> &'static str {
        self.relative_path
    }

    /// The final component of the [`Self::relative_path`]. Cost-free
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// The [`std::path::Path::file_stem`] part of the path: non-extension portion of [`std::path::Path::file_name`]. Cost-free
    pub fn stem(&self) -> &'static str {
        self.stem
    }
}
