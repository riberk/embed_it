use darling::FromMeta;
use regex::Regex;

use crate::fs::EntryPath;

#[derive(Debug, Clone)]
pub struct EntryRegex(Regex);

impl FromMeta for EntryRegex {
    fn from_string(value: &str) -> darling::Result<Self> {
        Regex::new(value)
            .map_err(|e| {
                darling::Error::custom(format!("'{}' is not a valid regex: {:#?}", value, e))
            })
            .map(Self)
    }
}

impl EntryRegex {
    pub fn is_match(&self, path: &EntryPath) -> bool {
        self.0.is_match(&path.relative)
    }
}

#[cfg(test)]
mod tests {
    use darling::FromMeta;
    use regex::Regex;
    use syn::parse_quote;

    use crate::fs::EntryPath;

    use super::EntryRegex;

    fn entry_path(relative: &str) -> EntryPath {
        EntryPath {
            origin: String::new(),
            relative: relative.to_owned(),
            ident: Default::default(),
            file_name: Default::default(),
            file_stem: Default::default(),
        }
    }
    #[test]
    fn is_match() {
        let pattern = EntryRegex(Regex::new(".+\\.txt$").unwrap());
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
            EntryRegex::from_meta(&parse_quote!(pattern = ".+\\.txt"))
                .unwrap()
                .0
                .as_str(),
            ".+\\.txt"
        );
    }

    #[test]
    fn from_meta_error() {
        EntryRegex::from_meta(&parse_quote!(pattern = "((")).unwrap_err();
    }
}
