mod eat;
mod next;
mod take;
mod till;
mod wrap;

pub use eat::*;
pub use next::*;
pub use take::*;
pub use till::*;

pub use wrap::{wrap, wrap_unchecked, Wrap, WrapUnchecked};

/// Combinators specific for parsing bytes.
pub mod bytes {
  pub use super::wrap::bytes::*;
}
