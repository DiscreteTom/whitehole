mod eat;
mod next;
mod take;
mod till;
mod wrap;

pub use eat::*;
pub use take::*;
pub use till::*;

pub use next::{next, Next};
pub use wrap::{wrap, wrap_unchecked, Wrap, WrapUnchecked};

/// Combinators specific for parsing bytes.
pub mod bytes {
  pub use super::next::bytes::*;
  pub use super::wrap::bytes::*;
}
