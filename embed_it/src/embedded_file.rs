use std::{fmt::Debug, path::Path};

#[derive(Clone, Copy)]
pub struct EmbeddedFile<'data> {
    path: &'data Path,
    content: &'data [u8],
}

impl<'data> EmbeddedFile<'data> {
    pub const fn new(path: &'data Path, content: &'data [u8]) -> Self {
        Self { path, content }
    }

    pub const fn content(&self) -> &'data [u8] {
        self.content
    }

    pub const fn path(&self) -> &'data Path {
        self.path
    }
}

impl<'data> Debug for EmbeddedFile<'data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { path, content } = self;

        let mut d = f.debug_struct("EmbeddedFile");

        d.field("path", path);
        d.field("content", &format!("<{} bytes>", content.len()));

        d.finish()
    }
}
