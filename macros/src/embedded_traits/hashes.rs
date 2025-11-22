pub mod ids;

#[cfg(feature = "any-hash")]
mod internal;

#[cfg(feature = "any-hash")]
pub use internal::*;
