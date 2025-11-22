pub mod ids;

#[cfg(feature = "any-compression")]
pub use internal::*;

#[cfg(feature = "any-compression")]
mod internal;
