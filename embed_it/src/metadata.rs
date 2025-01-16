use std::time::{Duration, SystemTime};

/// Metadata for a fs entry.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Metadata {
    accessed: Option<Duration>,
    created: Option<Duration>,
    modified: Option<Duration>,
}

impl Metadata {
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
