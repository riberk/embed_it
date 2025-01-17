use std::time::{Duration, SystemTime};

/// Metadata for a fs entry.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Metadata {
    /// Unix timestamp when the entry was last accessed.
    pub accessed: Option<Duration>,

    /// Unix timestamp when the entry was created.
    pub created: Option<Duration>,

    /// Unix timestamp when the entry was last modified.
    pub modified: Option<Duration>,
}

impl Metadata {
    /// Create new instance of [`Metadata`]
    pub const fn new(
        accessed: Option<Duration>,
        created: Option<Duration>,
        modified: Option<Duration>,
    ) -> Self {
        Metadata {
            accessed,
            created,
            modified,
        }
    }

    /// Get the time the entry was last accessed.
    ///
    /// Uses [`std::fs::Metadata::accessed()`].
    pub fn accessed(&self) -> Option<SystemTime> {
        self.accessed.map(|d| SystemTime::UNIX_EPOCH + d)
    }

    /// Get the time the entry was created.
    ///
    /// Uses [`std::fs::Metadata::accessed()`].
    pub fn created(&self) -> Option<SystemTime> {
        self.created.map(|d| SystemTime::UNIX_EPOCH + d)
    }

    /// Get the time the entry was last modified.
    ///
    /// Uses [`std::fs::Metadata::accessed()`].
    pub fn modified(&self) -> Option<SystemTime> {
        self.modified.map(|d| SystemTime::UNIX_EPOCH + d)
    }
}

#[cfg(test)]
mod tests {
    use crate::Metadata;
    use std::time::{Duration, SystemTime};

    #[test]
    fn getters() {
        let meta = Metadata::new(
            Some(Duration::from_secs(60)),
            Some(Duration::from_secs(120)),
            Some(Duration::from_secs(240)),
        );

        assert_eq!(
            meta.accessed(),
            Some(SystemTime::UNIX_EPOCH + meta.accessed.unwrap())
        );
        assert_eq!(
            meta.created(),
            Some(SystemTime::UNIX_EPOCH + meta.created.unwrap())
        );
        assert_eq!(
            meta.modified(),
            Some(SystemTime::UNIX_EPOCH + meta.modified.unwrap())
        );
    }
}
