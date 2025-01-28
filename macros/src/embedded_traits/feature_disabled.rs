#[derive(Debug, PartialEq, Eq, derive_more::Display, derive_more::Error)]
#[display("feature '{_0}' must be enabled")]
pub struct FeatureDisabled(#[error(not(source))] &'static str);

impl FeatureDisabled {
    pub fn md5() -> Self {
        Self("md5")
    }

    pub fn sha1() -> Self {
        Self("sha1")
    }

    pub fn sha2() -> Self {
        Self("sha2")
    }

    pub fn sha3() -> Self {
        Self("sha3")
    }

    pub fn blake3() -> Self {
        Self("blake3")
    }
}
