mod accepted;
mod debug;
mod flow;
mod state;
mod value;

pub use accepted::*;
pub use debug::*;
pub use flow::*;
pub use state::*;
pub use value::*;

macro_rules! create_simple_decorator {
  ($name:ident, $usage:literal) => {
    #[doc = $usage]
    #[derive(Copy, Clone, Debug)]
    pub struct $name<T> {
      action: T,
    }

    impl<T> $name<T> {
      #[inline]
      const fn new(action: T) -> Self {
        Self { action }
      }
    }
  };
}
// https://github.com/rust-lang/rust-clippy/issues/12808
#[allow(clippy::useless_attribute)]
#[allow(clippy::needless_pub_self)]
pub(self) use create_simple_decorator;

macro_rules! create_decorator {
  ($name:ident, $usage:literal, ($($derives:ident),*)) => {
    #[doc = $usage]
    #[derive(Copy, Clone, $($derives),*)]
    pub struct $name<T, D> {
      action: T,
      inner: D,
    }

    impl<T, D> $name<T, D> {
      #[inline]
      const fn new(action:T, inner: D) -> Self {
        Self {
          action,
          inner,
        }
      }
    }
  };
}
// https://github.com/rust-lang/rust-clippy/issues/12808
#[allow(clippy::useless_attribute)]
#[allow(clippy::needless_pub_self)]
pub(self) use create_decorator;

macro_rules! create_value_decorator {
  ($name:ident, $usage:literal) => {
    $crate::combinator::decorator::create_decorator!($name, $usage, (Debug));
  };
}
// https://github.com/rust-lang/rust-clippy/issues/12808
#[allow(clippy::useless_attribute)]
#[allow(clippy::needless_pub_self)]
pub(self) use create_value_decorator;

macro_rules! create_closure_decorator {
  ($name:ident, $usage:literal) => {
    $crate::combinator::decorator::create_decorator!($name, $usage, ());

    impl<T: core::fmt::Debug, D> core::fmt::Debug for $name<T, D> {
      #[inline]
      fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let action = &self.action;
        f.debug_struct(stringify!($name))
          .field(stringify!(action), action)
          .finish()
      }
    }
  };
}
// https://github.com/rust-lang/rust-clippy/issues/12808
#[allow(clippy::useless_attribute)]
#[allow(clippy::needless_pub_self)]
pub(self) use create_closure_decorator;
