use crate::{embedded_dir::EmbeddedDir, embedded_file::EmbeddedFile};

#[derive(Debug, Clone, Copy)]
pub enum EmbeddedEntry<'reference, 'data> {
    File(&'reference EmbeddedFile<'data>),
    Dir(&'reference dyn EmbeddedDir<'data>),
}

impl<'reference, 'data> EmbeddedEntry<'reference, 'data> {
    pub fn as_dir(&self) -> Option<&'reference dyn EmbeddedDir<'data>> {
        match self {
            EmbeddedEntry::File(_) => None,
            EmbeddedEntry::Dir(d) => Some(*d),
        }
    }

    pub fn as_file(&self) -> Option<&'reference EmbeddedFile<'data>> {
        match self {
            EmbeddedEntry::File(f) => Some(*f),
            EmbeddedEntry::Dir(_) => None,
        }
    }
}

impl<'reference, 'data> From<&'reference EmbeddedFile<'data>> for EmbeddedEntry<'reference, 'data> {
    fn from(value: &'reference EmbeddedFile<'data>) -> Self {
        Self::File(value)
    }
}

impl<'reference, 'data> From<&'reference dyn EmbeddedDir<'data>>
    for EmbeddedEntry<'reference, 'data>
{
    fn from(value: &'reference dyn EmbeddedDir<'data>) -> Self {
        Self::Dir(value)
    }
}
