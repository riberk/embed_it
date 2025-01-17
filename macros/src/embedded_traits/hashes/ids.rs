#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct AlgId {
    pub id: &'static str,
    pub feature: &'static str,
}

impl AlgId {
    pub const fn new(id: &'static str, feature: &'static str) -> Self {
        Self { id, feature }
    }
}

pub const MD5: &AlgId = &AlgId::new("Hash(md5)", "md5");
pub const SHA1: &AlgId = &AlgId::new("Hash(sha1)", "sha1");
pub const SHA2_224: &AlgId = &AlgId::new("Hash(sha2-224)", "sha2");
pub const SHA2_256: &AlgId = &AlgId::new("Hash(sha2-256)", "sha2");
pub const SHA2_384: &AlgId = &AlgId::new("Hash(sha2-384)", "sha2");
pub const SHA2_512: &AlgId = &AlgId::new("Hash(sha2-512)", "sha2");
pub const SHA3_224: &AlgId = &AlgId::new("Hash(sha3-224)", "sha3");
pub const SHA3_256: &AlgId = &AlgId::new("Hash(sha3-256)", "sha3");
pub const SHA3_384: &AlgId = &AlgId::new("Hash(sha3-384)", "sha3");
pub const SHA3_512: &AlgId = &AlgId::new("Hash(sha3-512)", "sha3");
pub const BLAKE3: &AlgId = &AlgId::new("Hash(blake3)", "blake3");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_features() {
        assert_eq!(MD5.feature, "md5");
        assert_eq!(SHA1.feature, "sha1");
        assert_eq!(SHA2_224.feature, "sha2");
        assert_eq!(SHA2_256.feature, "sha2");
        assert_eq!(SHA2_384.feature, "sha2");
        assert_eq!(SHA2_512.feature, "sha2");
        assert_eq!(SHA3_224.feature, "sha3");
        assert_eq!(SHA3_256.feature, "sha3");
        assert_eq!(SHA3_384.feature, "sha3");
        assert_eq!(SHA3_512.feature, "sha3");
        assert_eq!(BLAKE3.feature, "blake3");
    }
}
