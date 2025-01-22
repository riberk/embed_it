use darling::FromMeta;

use crate::embed::bool_like_enum::BoolLikeEnum;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GlobalField {
    #[default]
    No = 0,
    Yes = 1,
}

impl BoolLikeEnum for GlobalField {
    fn yes() -> Self {
        Self::Yes
    }

    fn no() -> Self {
        Self::No
    }
}

impl FromMeta for GlobalField {
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

    use super::GlobalField;

    #[test]
    fn from_meta() {
        assert_eq!(
            GlobalField::from_meta(&parse_quote!(value = true)).unwrap(),
            GlobalField::Yes
        );
        assert_eq!(
            GlobalField::from_meta(&parse_quote!(value)).unwrap(),
            GlobalField::Yes
        );
        assert_eq!(
            GlobalField::from_meta(&parse_quote!(value = 'y')).unwrap(),
            GlobalField::Yes
        );
        assert_eq!(
            GlobalField::from_meta(&parse_quote!(value = 't')).unwrap(),
            GlobalField::Yes
        );
        assert_eq!(
            GlobalField::from_meta(&parse_quote!(value = "y")).unwrap(),
            GlobalField::Yes
        );
        assert_eq!(
            GlobalField::from_meta(&parse_quote!(value = "t")).unwrap(),
            GlobalField::Yes
        );
        assert_eq!(
            GlobalField::from_meta(&parse_quote!(value = "true")).unwrap(),
            GlobalField::Yes
        );
        assert_eq!(
            GlobalField::from_meta(&parse_quote!(value = "yes")).unwrap(),
            GlobalField::Yes
        );

        assert_eq!(
            GlobalField::from_meta(&parse_quote!(value = false)).unwrap(),
            GlobalField::No
        );
        assert_eq!(
            GlobalField::from_meta(&parse_quote!(value = 'n')).unwrap(),
            GlobalField::No
        );
        assert_eq!(
            GlobalField::from_meta(&parse_quote!(value = 'f')).unwrap(),
            GlobalField::No
        );
        assert_eq!(
            GlobalField::from_meta(&parse_quote!(value = "n")).unwrap(),
            GlobalField::No
        );
        assert_eq!(
            GlobalField::from_meta(&parse_quote!(value = "f")).unwrap(),
            GlobalField::No
        );
        assert_eq!(
            GlobalField::from_meta(&parse_quote!(value = "no")).unwrap(),
            GlobalField::No
        );
        assert_eq!(
            GlobalField::from_meta(&parse_quote!(value = "false")).unwrap(),
            GlobalField::No
        );
    }

    #[test]
    fn from_meta_unsupported_str() {
        let value = "sefsfsf";
        let err = GlobalField::from_meta(&parse_quote!(value = #value)).unwrap_err();
        let err_str = format!("{err:?}");
        assert!(
            err_str.contains(value),
            "Unable to find actual value '{value}' in error message"
        );
    }

    #[test]
    fn from_meta_unsupported_char() {
        let value = '\u{1f600}';
        let err = GlobalField::from_meta(&parse_quote!(value = #value)).unwrap_err();
        let err_str = format!("{err:?}");
        assert!(
            err_str.contains(value),
            "Unable to find actual value '{value}' in error message"
        );
    }
}
