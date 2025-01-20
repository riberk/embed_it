use darling::FromMeta;

use crate::embed::bool_like_enum::BoolLikeEnum;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DeriveDefaultTraits {
    No = 0,
    #[default]
    Yes = 1,
}

impl BoolLikeEnum for DeriveDefaultTraits {
    fn yes() -> Self {
        Self::Yes
    }

    fn no() -> Self {
        Self::No
    }
}

impl FromMeta for DeriveDefaultTraits {
    fn from_bool(value: bool) -> darling::Result<Self> {
        Self::darling_from_bool(value)
    }

    fn from_char(value: char) -> darling::Result<Self> {
        Self::darling_from_char(value)
    }

    fn from_string(value: &str) -> darling::Result<Self> {
        Self::darling_from_string(value)
    }

    fn from_word() -> darling::Result<Self> {
        Ok(Self::Yes)
    }
}

#[cfg(test)]
mod tests {
    use darling::FromMeta;
    use syn::parse_quote;

    use super::DeriveDefaultTraits;

    #[test]
    fn from_meta() {
        assert_eq!(
            DeriveDefaultTraits::from_meta(&parse_quote!(value = true)).unwrap(),
            DeriveDefaultTraits::Yes
        );
        assert_eq!(
            DeriveDefaultTraits::from_meta(&parse_quote!(value)).unwrap(),
            DeriveDefaultTraits::Yes
        );
        assert_eq!(
            DeriveDefaultTraits::from_meta(&parse_quote!(value = 'y')).unwrap(),
            DeriveDefaultTraits::Yes
        );
        assert_eq!(
            DeriveDefaultTraits::from_meta(&parse_quote!(value = 't')).unwrap(),
            DeriveDefaultTraits::Yes
        );
        assert_eq!(
            DeriveDefaultTraits::from_meta(&parse_quote!(value = "y")).unwrap(),
            DeriveDefaultTraits::Yes
        );
        assert_eq!(
            DeriveDefaultTraits::from_meta(&parse_quote!(value = "t")).unwrap(),
            DeriveDefaultTraits::Yes
        );
        assert_eq!(
            DeriveDefaultTraits::from_meta(&parse_quote!(value = "true")).unwrap(),
            DeriveDefaultTraits::Yes
        );
        assert_eq!(
            DeriveDefaultTraits::from_meta(&parse_quote!(value = "yes")).unwrap(),
            DeriveDefaultTraits::Yes
        );

        assert_eq!(
            DeriveDefaultTraits::from_meta(&parse_quote!(value = false)).unwrap(),
            DeriveDefaultTraits::No
        );
        assert_eq!(
            DeriveDefaultTraits::from_meta(&parse_quote!(value = 'n')).unwrap(),
            DeriveDefaultTraits::No
        );
        assert_eq!(
            DeriveDefaultTraits::from_meta(&parse_quote!(value = 'f')).unwrap(),
            DeriveDefaultTraits::No
        );
        assert_eq!(
            DeriveDefaultTraits::from_meta(&parse_quote!(value = "n")).unwrap(),
            DeriveDefaultTraits::No
        );
        assert_eq!(
            DeriveDefaultTraits::from_meta(&parse_quote!(value = "f")).unwrap(),
            DeriveDefaultTraits::No
        );
        assert_eq!(
            DeriveDefaultTraits::from_meta(&parse_quote!(value = "no")).unwrap(),
            DeriveDefaultTraits::No
        );
        assert_eq!(
            DeriveDefaultTraits::from_meta(&parse_quote!(value = "false")).unwrap(),
            DeriveDefaultTraits::No
        );
    }

    #[test]
    fn from_meta_unsupported_str() {
        let value = "sefsfsf";
        let err = DeriveDefaultTraits::from_meta(&parse_quote!(value = #value)).unwrap_err();
        let err_str = format!("{err:?}");
        assert!(
            err_str.contains(value),
            "Unable to find actual value '{value}' in error message"
        );
    }

    #[test]
    fn from_meta_unsupported_char() {
        let value = '\u{1f600}';
        let err = DeriveDefaultTraits::from_meta(&parse_quote!(value = #value)).unwrap_err();
        let err_str = format!("{err:?}");
        assert!(
            err_str.contains(value),
            "Unable to find actual value '{value}' in error message"
        );
    }
}
