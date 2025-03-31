mod contextual;
mod eat;
mod next;
mod recur;
mod take;
mod till;
mod wrap;

pub use contextual::*;
pub use eat::*;
pub use next::*;
pub use recur::*;
pub use take::*;
pub use till::*;
pub use wrap::*;

pub mod bytes;

macro_rules! create_combinator {
  ($name:ident, $usage:literal, ($($derives:ident),*)) => {
    #[doc = $usage]
    #[derive(Copy, Clone, $($derives),*)]
    pub struct $name<T> {
      inner: T,
    }

    impl<T> $name<T> {
      /// Create a new instance.
      #[inline]
      pub const fn new(inner: T) -> Self {
        Self { inner }
      }
    }
  };
}
// https://github.com/rust-lang/rust-clippy/issues/12808
#[allow(clippy::useless_attribute)]
#[allow(clippy::needless_pub_self)]
pub(self) use create_combinator;

macro_rules! create_value_combinator {
  ($name:ident, $usage:literal) => {
    $crate::combinator::provided::create_combinator!($name, $usage, (Debug));
  };
}
// https://github.com/rust-lang/rust-clippy/issues/12808
#[allow(clippy::useless_attribute)]
#[allow(clippy::needless_pub_self)]
pub(self) use create_value_combinator;

macro_rules! create_closure_combinator {
  ($name:ident, $usage:literal) => {
    $crate::combinator::provided::create_combinator!($name, $usage, ());

    impl<T> core::fmt::Debug for $name<T> {
      #[inline]
      fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct(stringify!($name)).finish()
      }
    }
  };
}
// https://github.com/rust-lang/rust-clippy/issues/12808
#[allow(clippy::useless_attribute)]
#[allow(clippy::needless_pub_self)]
pub(self) use create_closure_combinator;
