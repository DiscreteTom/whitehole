//! Combinators for parsing bytes.

mod eat;
mod next;
mod take;
mod till;
mod wrap;
// TODO: recur

pub use eat::*;
pub use next::*;
pub use take::*;
pub use till::*;
pub use wrap::*;
