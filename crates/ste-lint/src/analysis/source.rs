#[path = "source_structure.rs"]
mod structure;

pub(crate) use super::canonical::CanonicalSource;
pub use super::canonical::CanonicalSpan;
pub(crate) use structure::*;
