//! The basic building block of a parser.
//!
//! Each [`Action`] is a small piece of parsing logic that
//! digest some bytes from the input, optionally change the state of the parsing,
//! and yield a value.
//!
//! For most cases, you don't need to use [`Action`] directly.
//! See [`Combinator`](crate::combinator::Combinator) and
//! [`Parser`](crate::parser::Parser) for higher-level APIs.
//!
//! # Stateless
//!
//! [`Action`]s are stateless and immutable,
//! but they can access external states to change their behavior.
//! See [`Context::state`] and [`Context::heap`] for more information.
//!
//! States are centrally managed by the parser,
//! so it's easy to realize peeking and backtracking.

mod context;
mod output;

use crate::instant::Instant;
use std::rc::Rc;

pub use context::*;
pub use output::*;

/// The basic building block of a parser.
/// See the [module level documentation](crate::action) for more information.
/// # Safety
/// The [`Output`] of [`Action::exec`] should satisfy the requirement of [`Output::digested`].
pub unsafe trait Action<Text: ?Sized = str> {
  /// See [`Output::value`].
  type Value;
  /// See [`Context::state`].
  type State;
  /// See [`Context::heap`].
  type Heap;

  /// Try to digest some bytes from the [`Instant::rest`],
  /// optionally change the state of the parsing,
  /// and yield a value.
  /// Return [`None`] to reject.
  fn exec(
    &self,
    instant: &Instant<&Text>,
    ctx: Context<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>>;
}

unsafe impl<Text: ?Sized, T: Action<Text> + ?Sized> Action<Text> for &T {
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    ctx: Context<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    (**self).exec(instant, ctx)
  }
}

unsafe impl<Text: ?Sized, T: Action<Text> + ?Sized> Action<Text> for Box<T> {
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    ctx: Context<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    self.as_ref().exec(instant, ctx)
  }
}

unsafe impl<Text: ?Sized, T: Action<Text> + ?Sized> Action<Text> for Rc<T> {
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    ctx: Context<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    self.as_ref().exec(instant, ctx)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{combinator::take, instant::Instant};

  fn assert_str_action(t: impl Action<Value = (), State = (), Heap = ()>) {
    assert!(t
      .exec(
        &Instant::new("123"),
        Context {
          state: &mut (),
          heap: &mut ()
        }
      )
      .is_some());
  }
  fn assert_bytes_action(t: impl Action<[u8], Value = (), State = (), Heap = ()>) {
    assert!(t
      .exec(
        &Instant::new(b"123"),
        Context {
          state: &mut (),
          heap: &mut ()
        }
      )
      .is_some());
  }

  #[test]
  fn action_ref() {
    assert_str_action(&take(1));
    assert_bytes_action(&take(1));
  }

  #[test]
  fn action_dyn_ref() {
    assert_str_action(&take(1) as &dyn Action<State = (), Heap = (), Value = ()>);
    assert_bytes_action(&take(1) as &dyn Action<[u8], State = (), Heap = (), Value = ()>);
  }

  #[test]
  fn boxed_action() {
    assert_str_action(Box::new(take(1)));
    assert_bytes_action(Box::new(take(1)));
  }

  #[test]
  fn boxed_dyn_action() {
    assert_str_action(Box::new(take(1)) as Box<dyn Action<State = (), Heap = (), Value = ()>>);
    assert_bytes_action(
      Box::new(take(1)) as Box<dyn Action<[u8], State = (), Heap = (), Value = ()>>
    );
  }

  #[test]
  fn rc_action() {
    assert_str_action(Rc::new(take(1)));
    assert_bytes_action(Rc::new(take(1)));
  }

  #[test]
  fn rc_dyn_action() {
    assert_str_action(Rc::new(take(1)) as Rc<dyn Action<State = (), Heap = (), Value = ()>>);
    assert_bytes_action(Rc::new(take(1)) as Rc<dyn Action<[u8], State = (), Heap = (), Value = ()>>);
  }
}
