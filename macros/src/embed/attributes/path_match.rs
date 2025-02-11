use crate::{
    embed::{pattern::EntryPattern, regex::EntryRegex},
    fs::EntryPath,
};

#[derive(Debug, Clone, darling::FromMeta, Default, PartialEq, Eq)]
pub struct PathMatch {
    #[darling(rename = "pattern")]
    pattern: Option<EntryPattern>,

    #[darling(rename = "regex")]
    regex: Option<EntryRegex>,
}

impl PathMatch {
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

impl From<EntryPattern> for PathMatch {
    fn from(value: EntryPattern) -> Self {
        Self {
            pattern: Some(value),
            regex: None,
        }
    }
}

impl From<EntryRegex> for PathMatch {
    fn from(value: EntryRegex) -> Self {
        Self {
            pattern: None,
            regex: Some(value),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PathMatchSet {
    /// matches any path
    Any,

    /// matches no path
    None,

    /// matches if one of an item matches
    OneOf(Vec<PathMatch>),
}

impl PathMatchSet {
    pub fn is_match(&self, path: &EntryPath) -> bool {
        match self {
            PathMatchSet::Any => true,
            PathMatchSet::None => false,
            PathMatchSet::OneOf(matches) => matches.iter().any(|v| v.is_match(path)),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PathMatcher {
    include: PathMatchSet,
    exclude: PathMatchSet,
}

impl PathMatcher {
    pub fn should_be_included(&self, path: &EntryPath) -> bool {
        if self.exclude.is_match(path) {
            false
        } else {
            self.include.is_match(path)
        }
    }
}

#[derive(Debug, darling::FromMeta, Clone, PartialEq, Eq, Default)]
pub struct PathMatcherAttr {
    #[darling(multiple, default, rename = "include")]
    include: Vec<PathMatch>,

    #[darling(multiple, default, rename = "exclude")]
    exclude: Vec<PathMatch>,
}

impl From<PathMatcherAttr> for PathMatcher {
    fn from(value: PathMatcherAttr) -> Self {
        Self {
            include: value
                .include
                .is_empty()
                .then_some(PathMatchSet::Any)
                .unwrap_or_else(|| PathMatchSet::OneOf(value.include)),
            exclude: value
                .exclude
                .is_empty()
                .then_some(PathMatchSet::None)
                .unwrap_or_else(|| PathMatchSet::OneOf(value.exclude)),
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

    use super::{PathMatch, PathMatchSet, PathMatcher, PathMatcherAttr};

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
        let include = PathMatch::from(EntryPattern::new(Pattern::new("*.txt").unwrap()));
        assert!(include.is_match(&entry_path("1.txt")));
        assert!(include.is_match(&entry_path("2.txt")));
        assert!(include.is_match(&entry_path("q/2.txt")));
        assert!(!include.is_match(&entry_path("q/2.aaa")));
        assert!(!include.is_match(&entry_path("q/2.txt.aaa")));
    }

    #[test]
    fn is_match_regex() {
        let include = PathMatch::from(EntryRegex::new(Regex::new(".*\\.txt$").unwrap()));
        assert!(include.is_match(&entry_path("1.txt")));
        assert!(include.is_match(&entry_path("2.txt")));
        assert!(include.is_match(&entry_path("q/2.txt")));
        assert!(!include.is_match(&entry_path("q/2.aaa")));
        assert!(!include.is_match(&entry_path("q/2.txt.aaa")));
    }

    #[test]
    fn is_match_none() {
        let include = PathMatch {
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
        let include = PathMatch {
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
        let include = PathMatch::from_meta(&parse_quote!(q())).unwrap();
        assert!(include.pattern.is_none());
        assert!(include.regex.is_none());
    }

    #[test]
    fn from_meta_pattern() {
        let include = PathMatch::from_meta(&parse_quote!(q(pattern = "123"))).unwrap();
        assert_eq!(include.pattern.unwrap().to_string(), "123".to_owned());
        assert!(include.regex.is_none());
    }

    #[test]
    fn from_meta_regex() {
        let include = PathMatch::from_meta(&parse_quote!(q(regex = "123"))).unwrap();
        assert_eq!(include.regex.unwrap().to_string(), "123".to_owned());
        assert!(include.pattern.is_none());
    }

    #[test]
    fn from_meta_both() {
        let include =
            PathMatch::from_meta(&parse_quote!(q(regex = "123", pattern = "123"))).unwrap();
        assert_eq!(include.regex.unwrap().to_string(), "123".to_owned());
        assert_eq!(include.pattern.unwrap().to_string(), "123".to_owned());
    }

    #[test]
    fn path_match_set_any_is_match() {
        assert!(PathMatchSet::Any.is_match(&entry_path("")));
        assert!(PathMatchSet::Any.is_match(&entry_path("aaaaaaaa")));
    }

    #[test]
    fn path_match_set_none_is_match() {
        assert!(!PathMatchSet::None.is_match(&entry_path("")));
        assert!(!PathMatchSet::None.is_match(&entry_path("aaaaaaaa")));
    }

    #[test]
    fn path_match_set_on_of_is_match() {
        let first = PathMatch::from(EntryPattern::new(Pattern::new("*.txt").unwrap()));
        let second = PathMatch::from(EntryPattern::new(Pattern::new("*.svg").unwrap()));
        let set = PathMatchSet::OneOf(vec![first, second]);
        assert!(!set.is_match(&entry_path("")));
        assert!(!set.is_match(&entry_path("aa.png")));
        assert!(set.is_match(&entry_path("aa.txt")));
        assert!(set.is_match(&entry_path("aa.svg")));
    }

    #[test]
    fn path_matcher_should_be_included_empty() {
        let matcher = PathMatcher {
            include: PathMatchSet::Any,
            exclude: PathMatchSet::None,
        };

        assert!(matcher.should_be_included(&entry_path("")));
        assert!(matcher.should_be_included(&entry_path("123")));
        assert!(matcher.should_be_included(&entry_path("11/2321/23")));
        assert!(matcher.should_be_included(&entry_path("q.txt")));
        assert!(matcher.should_be_included(&entry_path("q.txt.q2e")));
        assert!(matcher.should_be_included(&entry_path(".q.txt.q2e")));
    }

    #[test]
    fn path_matcher_should_be_included_include() {
        let first = PathMatch::from(EntryPattern::new(Pattern::new("*.txt").unwrap()));
        let second = PathMatch::from(EntryPattern::new(Pattern::new("*.svg").unwrap()));

        let matcher = PathMatcher {
            include: PathMatchSet::OneOf(vec![first, second]),
            exclude: PathMatchSet::None,
        };

        assert!(matcher.should_be_included(&entry_path("1.txt")));
        assert!(matcher.should_be_included(&entry_path("2.txt")));
        assert!(matcher.should_be_included(&entry_path("a.svg")));
        assert!(!matcher.should_be_included(&entry_path("b.sv")));
        assert!(matcher.should_be_included(&entry_path("1/2/3.svg")));
        assert!(!matcher.should_be_included(&entry_path("aaa")));
    }

    #[test]
    fn path_matcher_should_be_included_exclude() {
        let first = PathMatch::from(EntryPattern::new(Pattern::new("*.txt").unwrap()));
        let second = PathMatch::from(EntryPattern::new(Pattern::new("*.svg").unwrap()));

        let matcher = PathMatcher {
            include: PathMatchSet::Any,
            exclude: PathMatchSet::OneOf(vec![first, second]),
        };

        assert!(!matcher.should_be_included(&entry_path("1.txt")));
        assert!(!matcher.should_be_included(&entry_path("2.txt")));
        assert!(!matcher.should_be_included(&entry_path("a.svg")));
        assert!(matcher.should_be_included(&entry_path("b.sv")));
        assert!(!matcher.should_be_included(&entry_path("1/2/3.svg")));
        assert!(matcher.should_be_included(&entry_path("aaa")));
    }

    #[test]
    fn path_matcher_should_be_included_include_exclude() {
        let exclude_first = PathMatch::from(EntryPattern::new(Pattern::new("*.txt").unwrap()));
        let exclude_second = PathMatch::from(EntryPattern::new(Pattern::new("*.svg").unwrap()));
        let include = PathMatch::from(EntryPattern::new(Pattern::new("a*").unwrap()));
        let matcher = PathMatcher {
            include: PathMatchSet::OneOf(vec![include]),
            exclude: PathMatchSet::OneOf(vec![exclude_first, exclude_second]),
        };

        assert!(!matcher.should_be_included(&entry_path("1.txt")));
        assert!(!matcher.should_be_included(&entry_path("2.txt")));
        assert!(!matcher.should_be_included(&entry_path("a.svg")));
        assert!(matcher.should_be_included(&entry_path("a.sv")));
        assert!(!matcher.should_be_included(&entry_path("1/2/3.svg")));
        assert!(matcher.should_be_included(&entry_path("aaa")));
        assert!(!matcher.should_be_included(&entry_path("baa")));
        assert!(!matcher.should_be_included(&entry_path("b.a")));
    }

    #[test]
    fn path_matcher_from_attr_empty() {
        let attr = PathMatcherAttr {
            include: Vec::from([]),
            exclude: Vec::from([]),
        };

        let matcher = PathMatcher::from(attr);

        assert_eq!(
            matcher,
            PathMatcher {
                include: PathMatchSet::Any,
                exclude: PathMatchSet::None,
            }
        );
    }

    #[test]
    fn path_matcher_from_attr_both() {
        let attr = PathMatcherAttr {
            include: Vec::from([
                PathMatch::from(EntryPattern::new(Pattern::new("*.txt").unwrap())),
                PathMatch::from(EntryPattern::new(Pattern::new("*.svg").unwrap())),
            ]),
            exclude: Vec::from([
                PathMatch::from(EntryPattern::new(Pattern::new("*.svg.txt").unwrap())),
                PathMatch::from(EntryPattern::new(Pattern::new("*.txt.svg").unwrap())),
            ]),
        };

        let matcher = PathMatcher::from(attr.clone());

        assert_eq!(
            matcher,
            PathMatcher {
                include: PathMatchSet::OneOf(attr.include),
                exclude: PathMatchSet::OneOf(attr.exclude),
            }
        );
    }

    #[test]
    fn path_matcher_attr_from_meta_empty() {
        let attr = PathMatcherAttr::from_meta(&parse_quote!(i(
            include(pattern = "*.txt"),
            include(regex = ".*\\.svg"),
            exclude(pattern = "private"),
        )))
        .unwrap();
        assert_eq!(
            attr,
            PathMatcherAttr {
                include: Vec::from([
                    PathMatch::from(EntryPattern::new(Pattern::new("*.txt").unwrap())),
                    PathMatch::from(EntryRegex::new(Regex::new(".*\\.svg").unwrap())),
                ]),
                exclude: Vec::from([PathMatch::from(EntryPattern::new(
                    Pattern::new("private").unwrap()
                )),])
            }
        );
    }
}
