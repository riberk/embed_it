use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Entry<Dir, File = Dir> {
    Dir(Dir),
    File(File),
}

impl<Dir, File> Entry<Dir, File> {
    pub fn as_ref(&self) -> Entry<&Dir, &File> {
        match self {
            Entry::Dir(e) => Entry::Dir(e),
            Entry::File(e) => Entry::File(e),
        }
    }

    pub fn file(self) -> Option<File> {
        match self {
            Entry::Dir(_) => None,
            Entry::File(f) => Some(f),
        }
    }

    pub fn dir(self) -> Option<Dir> {
        match self {
            Entry::Dir(d) => Some(d),
            Entry::File(_) => None,
        }
    }

    pub fn kind(&self) -> EntryKind {
        match self {
            Entry::Dir(_) => EntryKind::Dir,
            Entry::File(_) => EntryKind::File,
        }
    }

    pub fn map<T, U>(
        self,
        map_dir: impl FnOnce(Dir) -> T,
        map_file: impl FnOnce(File) -> U,
    ) -> Entry<T, U> {
        match self {
            Entry::Dir(d) => Entry::Dir(map_dir(d)),
            Entry::File(f) => Entry::File(map_file(f)),
        }
    }
}

impl<T> Entry<T, T> {
    pub fn value(self) -> T {
        match self {
            Entry::Dir(e) => e,
            Entry::File(e) => e,
        }
    }

    pub fn map_value<U>(self, map: impl FnOnce(T) -> U) -> Entry<U, U> {
        match self {
            Entry::Dir(d) => Entry::Dir(map(d)),
            Entry::File(f) => Entry::File(map(f)),
        }
    }
}

impl<Dir: PartialOrd, File: PartialOrd> PartialOrd for Entry<Dir, File> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Dir(_), Self::File(_)) => Some(Ordering::Less),
            (Self::File(_), Self::Dir(_)) => Some(Ordering::Greater),
            (Self::Dir(left), Self::Dir(right)) => left.partial_cmp(right),
            (Self::File(left), Self::File(right)) => left.partial_cmp(right),
        }
    }
}

impl<Dir: Ord, File: Ord> Ord for Entry<Dir, File> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EntryKind {
    Dir,
    File,
}

impl PartialOrd for EntryKind {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EntryKind {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (EntryKind::Dir, EntryKind::File) => Ordering::Less,
            (EntryKind::File, EntryKind::Dir) => Ordering::Greater,
            _ => Ordering::Equal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering::*;

    fn dir<T>(v: T) -> Entry<T> {
        Entry::Dir(v)
    }

    fn file<T>(v: T) -> Entry<T> {
        Entry::File(v)
    }

    #[test]
    fn ord() {
        assert_eq!(dir(2).cmp(&dir(1)), Greater);
        assert_eq!(dir(1).cmp(&dir(2)), Less);
        assert_eq!(dir(1).cmp(&dir(1)), Equal);

        assert_eq!(file(2).cmp(&file(1)), Greater);
        assert_eq!(file(1).cmp(&file(2)), Less);
        assert_eq!(file(1).cmp(&file(1)), Equal);

        assert_eq!(file(1).cmp(&dir(1)), Greater);
        assert_eq!(dir(1).cmp(&file(1)), Less);
        assert_eq!(file(1).cmp(&dir(100)), Greater);
        assert_eq!(dir(100).cmp(&file(1)), Less);
        assert_eq!(file(100).cmp(&dir(1)), Greater);
        assert_eq!(dir(1).cmp(&file(100)), Less);
    }

    #[test]
    fn partial_ord() {
        assert_eq!(dir(2).partial_cmp(&dir(1)), Some(Greater));
        assert_eq!(dir(1).partial_cmp(&dir(2)), Some(Less));
        assert_eq!(dir(1).partial_cmp(&dir(1)), Some(Equal));

        assert_eq!(file(2).partial_cmp(&file(1)), Some(Greater));
        assert_eq!(file(1).partial_cmp(&file(2)), Some(Less));
        assert_eq!(file(1).partial_cmp(&file(1)), Some(Equal));

        assert_eq!(file(1).partial_cmp(&dir(1)), Some(Greater));
        assert_eq!(dir(1).partial_cmp(&file(1)), Some(Less));
        assert_eq!(file(1).partial_cmp(&dir(100)), Some(Greater));
        assert_eq!(dir(100).partial_cmp(&file(1)), Some(Less));
        assert_eq!(file(100).partial_cmp(&dir(1)), Some(Greater));
        assert_eq!(dir(1).partial_cmp(&file(100)), Some(Less));
    }

    #[test]
    fn kind_ord() {
        assert_eq!(EntryKind::Dir.cmp(&EntryKind::File), Less);
        assert_eq!(EntryKind::File.cmp(&EntryKind::Dir), Greater);
        assert_eq!(EntryKind::File.cmp(&EntryKind::File), Equal);
        assert_eq!(EntryKind::Dir.cmp(&EntryKind::Dir), Equal);

        assert_eq!(EntryKind::Dir.partial_cmp(&EntryKind::File), Some(Less));
        assert_eq!(EntryKind::File.partial_cmp(&EntryKind::Dir), Some(Greater));
        assert_eq!(EntryKind::File.partial_cmp(&EntryKind::File), Some(Equal));
        assert_eq!(EntryKind::Dir.partial_cmp(&EntryKind::Dir), Some(Equal));
    }

    #[test]
    fn to_option() {
        assert!(dir(1).file().is_none());
        assert!(file(1).dir().is_none());

        assert_eq!(dir(1).dir(), Some(1));
        assert_eq!(file(1).file(), Some(1));
    }

    #[test]
    fn map() {
        assert_eq!(dir(1).map(|i| i * 2, |i| i * 3).value(), 2);
        assert_eq!(file(1).map(|i| i * 2, |i| i * 3).value(), 3);
    }

    #[test]
    fn map_value() {
        assert_eq!(dir(1).map_value(|i| i * 2).value(), 2);
        assert_eq!(file(1).map_value(|i| i * 2).value(), 2);
    }
}
