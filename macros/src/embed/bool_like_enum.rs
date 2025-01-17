use std::fmt::Display;

pub trait BoolLikeEnum: Sized + Eq {
    const HELP_MESSAGE: &str = r#"You should use true and false literals (without quotes), chars 'y', 't', 'n', 'f', or strings "y", "t", "yes", "true", "n", "f", "no", "false""#;
    fn yes() -> Self;
    fn no() -> Self;

    fn as_bool(&self) -> bool {
        self == &Self::yes()
    }

    fn error(v: impl Display) -> darling::Error {
        darling::Error::custom(format!(
            "Unable to parse value '{v}'.{}",
            Self::HELP_MESSAGE
        ))
    }

    fn darling_from_bool(value: bool) -> darling::Result<Self> {
        match value {
            true => Ok(Self::yes()),
            false => Ok(Self::no()),
        }
    }

    fn darling_from_char(value: char) -> darling::Result<Self> {
        match value {
            'y' | 't' => Self::darling_from_bool(true),
            'f' | 'n' => Self::darling_from_bool(false),
            _ => Err(Self::error(value)),
        }
    }

    fn darling_from_string(value: &str) -> darling::Result<Self> {
        match value {
            "yes" | "true" | "t" | "y" => Self::darling_from_bool(true),
            "no" | "false" | "f" | "n" => Self::darling_from_bool(false),
            _ => Err(Self::error(value)),
        }
    }
}
