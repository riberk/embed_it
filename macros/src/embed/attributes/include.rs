use crate::{
    embed::{pattern::EntryPattern, regex::EntryRegex},
    fs::EntryPath,
};

#[derive(Debug, Clone, darling::FromMeta, Default)]
pub struct Include {
    #[darling(rename = "pattern")]
    pattern: Option<EntryPattern>,

    #[darling(rename = "regex")]
    regex: Option<EntryRegex>,
}

impl Include {
    pub fn is_match(&self, path: &EntryPath) -> bool {
        self.regex
            .as_ref()
            .map(|v| v.is_match(path))
            .unwrap_or(true)
            && self
                .pattern
                .as_ref()
                .map(|v| v.is_match(path))
                .unwrap_or(true)
    }
    
    #[cfg(test)]
    pub fn regex(&self) -> Option<&EntryRegex> {
        self.regex.as_ref()
    }
    
    #[cfg(test)]
    pub fn pattern(&self) -> Option<&EntryPattern> {
        self.pattern.as_ref()
    }
}

impl From<EntryPattern> for Include {
    fn from(value: EntryPattern) -> Self {
        Self {
            pattern: Some(value),
            regex: None,
        }
    }
}

impl From<EntryRegex> for Include {
    fn from(value: EntryRegex) -> Self {
        Self {
            pattern: None,
            regex: Some(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use darling::FromMeta;
    use glob::Pattern;
    use regex::Regex;
    use syn::parse_quote;

    use crate::{
        embed::{pattern::EntryPattern, regex::EntryRegex},
        fs::{EntryIdent, EntryPath},
    };

    use super::Include;

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
    fn is_match_pattern() {
        let include = Include::from(EntryPattern::new(Pattern::new("*.txt").unwrap()));
        assert!(include.is_match(&entry_path("1.txt")));
        assert!(include.is_match(&entry_path("2.txt")));
        assert!(include.is_match(&entry_path("q/2.txt")));
        assert!(!include.is_match(&entry_path("q/2.aaa")));
        assert!(!include.is_match(&entry_path("q/2.txt.aaa")));
    }

    #[test]
    fn is_match_regex() {
        let include = Include::from(EntryRegex::new(Regex::new(".*\\.txt$").unwrap()));
        assert!(include.is_match(&entry_path("1.txt")));
        assert!(include.is_match(&entry_path("2.txt")));
        assert!(include.is_match(&entry_path("q/2.txt")));
        assert!(!include.is_match(&entry_path("q/2.aaa")));
        assert!(!include.is_match(&entry_path("q/2.txt.aaa")));
    }

    #[test]
    fn is_match_none() {
        let include = Include {
            pattern: None,
            regex: None,
        };
        assert!(include.is_match(&entry_path("1.txt")));
        assert!(include.is_match(&entry_path("2.txt")));
        assert!(include.is_match(&entry_path("q/2.txt")));
        assert!(include.is_match(&entry_path("q/2.aaa")));
        assert!(include.is_match(&entry_path("q/2.txt.aaa")));
        assert!(include.is_match(&entry_path("")));
    }

    #[test]
    fn is_match_both() {
        let include = Include {
            pattern: Some(EntryPattern::new(Pattern::new("*.txt").unwrap())),
            regex: Some(EntryRegex::new(Regex::new(".*\\.aaa\\..+").unwrap())),
        };
        assert!(!include.is_match(&entry_path("1.txt")));
        assert!(!include.is_match(&entry_path("2.txt")));
        assert!(!include.is_match(&entry_path("q/2.txt")));
        assert!(!include.is_match(&entry_path("q/2.aaa")));
        assert!(include.is_match(&entry_path("q/2.aaa.txt")));
        assert!(!include.is_match(&entry_path("")));
        assert!(!include.is_match(&entry_path("aaa.aaa.ttt")));
    }

    #[test]
    fn from_meta_empty() {
        let include = Include::from_meta(&parse_quote!(q())).unwrap();
        assert!(include.pattern.is_none());
        assert!(include.regex.is_none());
    }

    #[test]
    fn from_meta_pattern() {
        let include = Include::from_meta(&parse_quote!(q(pattern = "123"))).unwrap();
        assert_eq!(include.pattern.unwrap().to_string(), "123".to_owned());
        assert!(include.regex.is_none());
    }

    #[test]
    fn from_meta_regex() {
        let include = Include::from_meta(&parse_quote!(q(regex = "123"))).unwrap();
        assert_eq!(include.regex.unwrap().to_string(), "123".to_owned());
        assert!(include.pattern.is_none());
    }

    #[test]
    fn from_meta_both() {
        let include = Include::from_meta(&parse_quote!(q(regex = "123", pattern = "123"))).unwrap();
        assert_eq!(include.regex.unwrap().to_string(), "123".to_owned());
        assert_eq!(include.pattern.unwrap().to_string(), "123".to_owned());
    }
}
