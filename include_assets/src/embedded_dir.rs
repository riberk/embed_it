use std::{collections::HashMap, fmt::Debug, path::Path};

use crate::embedded_entry::EmbeddedEntry;

pub trait EmbeddedDir<'data>: Debug {
    fn path(&self) -> &'data Path;
    fn entries(&self) -> DirEntries<'_, 'data>;
    fn index(&self) -> &HashMap<&'data Path, usize>;

    fn get(&self, path: &Path) -> Option<EmbeddedEntry<'_, 'data>>;
    fn by_index(&self, idx: usize) -> Option<EmbeddedEntry<'_, 'data>>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait Instance<'data> {
    fn instance() -> &'static Self;
}

pub fn get_from_dir<'reference, 'data>(
    dir: &'reference dyn EmbeddedDir<'data>,
    path: &Path,
) -> Option<EmbeddedEntry<'reference, 'data>> {
    let mut dir = Some(dir);
    let mut entry = None;
    for component in path.iter() {
        match dir {
            Some(d) => {
                entry = d
                    .index()
                    .get(Path::new(component))
                    .and_then(|&idx| d.by_index(idx));
                dir = entry.and_then(|v| v.as_dir());
            }
            None => return None,
        }
    }
    entry
}

pub struct DirEntries<'reference, 'data> {
    dir: &'reference dyn EmbeddedDir<'data>,
}

impl<'reference, 'data> DirEntries<'reference, 'data> {
    pub fn new(dir: &'reference dyn EmbeddedDir<'data>) -> Self {
        Self { dir }
    }

    pub fn iter(&self) -> Entries<'_, 'data> {
        Entries::new(self.dir)
    }
}

impl<'reference, 'data> IntoIterator for DirEntries<'reference, 'data> {
    type Item = EmbeddedEntry<'reference, 'data>;
    type IntoIter = Entries<'reference, 'data>;

    fn into_iter(self) -> Self::IntoIter {
        Entries::new(self.dir)
    }
}

pub struct Entries<'reference, 'data> {
    dir: &'reference dyn EmbeddedDir<'data>,
    idx: usize,
}

impl<'reference, 'data> Entries<'reference, 'data> {
    pub fn new(dir: &'reference dyn EmbeddedDir<'data>) -> Self {
        Self { dir, idx: 0 }
    }

    fn len(&self) -> usize {
        self.dir.len() - self.idx
    }
}

impl<'reference, 'data> Iterator for Entries<'reference, 'data> {
    type Item = EmbeddedEntry<'reference, 'data>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.dir.len() {
            None
        } else {
            let res = self.dir.by_index(self.idx);
            self.idx += 1;
            res
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        if self.idx >= self.dir.len() {
            None
        } else {
            self.dir.by_index(self.dir.len() - 1)
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.idx += n;
        self.next()
    }
}

impl<'dir, 'reference> ExactSizeIterator for Entries<'dir, 'reference> {}
