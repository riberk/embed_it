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

pub const GZIP: &AlgId = &AlgId::new("Compression(gzip)", "gzip");
pub const ZSTD: &AlgId = &AlgId::new("Compression(zstd)", "zstd");
pub const BROTLI: &AlgId = &AlgId::new("Compression(brotli)", "brotli");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_features() {
        assert_eq!(GZIP.feature, "gzip");
        assert_eq!(ZSTD.feature, "zstd");
        assert_eq!(BROTLI.feature, "brotli");
    }
}
