use darling::FromMeta;
use glob::Pattern;

use crate::fs::EntryPath;

#[derive(Debug, Clone, derive_more::Display, PartialEq, Eq)]
pub struct EntryPattern(Pattern);

impl FromMeta for EntryPattern {
    fn from_string(value: &str) -> darling::Result<Self> {
        Pattern::new(value)
            .map_err(|e| {
                darling::Error::custom(format!("'{}' is not a valid glob pattern: {:#?}", value, e))
            })
            .map(Self)
    }
}

impl EntryPattern {
    #[cfg(test)]
    pub fn new(pattern: Pattern) -> Self {
        Self(pattern)
    }

    pub fn is_match(&self, path: &EntryPath) -> bool {
        self.0.matches_path(path.relative_path())
    }
}

#[cfg(test)]
mod tests {
    use darling::FromMeta;
    use glob::Pattern;
    use syn::parse_quote;

    use crate::fs::{EntryIdent, EntryPath};

    use super::EntryPattern;

    fn entry_path(relative: &str) -> EntryPath {
        EntryPath {
            origin: String::new(),
            relative: relative.to_owned(),
            ident: EntryIdent::root(parse_quote!(Assets)),
            file_name: Default::default(),
            file_stem: Default::default(),
        }
    }
    #[test]
    fn is_match() {
        let pattern = EntryPattern(Pattern::new("*.txt").unwrap());
        assert!(pattern.is_match(&entry_path("1.txt")));
        assert!(pattern.is_match(&entry_path("abc/1.txt")));
        assert!(pattern.is_match(&entry_path("./1.txt")));
        assert!(pattern.is_match(&entry_path("qqqwdaeff1.txt")));
        assert!(!pattern.is_match(&entry_path("qqqwdaeff1.tx")));
        assert!(!pattern.is_match(&entry_path("qqqwdaeff1.txt.123")));
    }

    #[test]
    fn from_meta() {
        assert_eq!(
            EntryPattern::from_meta(&parse_quote!(pattern = "*.txt"))
                .unwrap()
                .0
                .as_str(),
            "*.txt"
        );
    }

    #[test]
    fn from_meta_error() {
        EntryPattern::from_meta(&parse_quote!(pattern = "**.txt")).unwrap_err();
    }
}
