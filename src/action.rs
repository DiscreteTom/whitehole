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
//! See [`Input::state`] and [`Input::heap`] for more information.
//!
//! States are centrally managed by the parser,
//! so it's easy to realize peeking and backtracking.

mod input;
mod output;

use crate::instant::Instant;
use std::rc::Rc;

pub use input::*;
pub use output::*;

/// The basic building block of a parser.
/// See the [module level documentation](crate::action) for more information.
/// # Safety
/// The [`Output`] of [`Action::exec`] should satisfy the requirement of [`Output::digested`].
pub unsafe trait Action {
  /// See [`Instant::text`].
  type Text: ?Sized;
  /// See [`Input::state`].
  type State;
  /// See [`Input::heap`].
  type Heap;
  /// See [`Output::value`].
  type Value;

  /// Try to digest some bytes from the [`Instant::rest`],
  /// optionally change the state of the parsing,
  /// and yield a value.
  /// Return [`None`] to reject.
  fn exec(
    &self,
    input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>>;
}

unsafe impl<T: Action + ?Sized> Action for &T {
  type Text = T::Text;
  type State = T::State;
  type Heap = T::Heap;
  type Value = T::Value;

  #[inline]
  fn exec(
    &self,
    input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    (**self).exec(input)
  }
}

unsafe impl<T: Action + ?Sized> Action for Box<T> {
  type Text = T::Text;
  type State = T::State;
  type Heap = T::Heap;
  type Value = T::Value;

  #[inline]
  fn exec(
    &self,
    input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    self.as_ref().exec(input)
  }
}

unsafe impl<T: Action + ?Sized> Action for Rc<T> {
  type Text = T::Text;
  type State = T::State;
  type Heap = T::Heap;
  type Value = T::Value;

  #[inline]
  fn exec(
    &self,
    input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    self.as_ref().exec(input)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    combinator::{bytes, take},
    instant::Instant,
  };

  fn assert_str_action(t: impl Action<Text = str, State = (), Heap = (), Value = ()>) {
    assert!(t
      .exec(Input {
        instant: &Instant::new("123"),
        state: &mut (),
        heap: &mut ()
      })
      .is_some());
  }
  fn assert_bytes_action(t: impl Action<Text = [u8], State = (), Heap = (), Value = ()>) {
    assert!(t
      .exec(Input {
        instant: &Instant::new(b"123"),
        state: &mut (),
        heap: &mut ()
      })
      .is_some());
  }

  #[test]
  fn action_ref() {
    assert_str_action(&take(1));
    assert_bytes_action(&bytes::take(1));
  }

  #[test]
  fn action_dyn_ref() {
    assert_str_action(&take(1) as &dyn Action<Text = str, State = (), Heap = (), Value = ()>);
    assert_bytes_action(
      &bytes::take(1) as &dyn Action<Text = [u8], State = (), Heap = (), Value = ()>
    );
  }

  #[test]
  fn boxed_action() {
    assert_str_action(Box::new(take(1)));
    assert_bytes_action(Box::new(bytes::take(1)));
  }

  #[test]
  fn boxed_dyn_action() {
    assert_str_action(
      Box::new(take(1)) as Box<dyn Action<Text = str, State = (), Heap = (), Value = ()>>
    );
    assert_bytes_action(
      Box::new(bytes::take(1)) as Box<dyn Action<Text = [u8], State = (), Heap = (), Value = ()>>
    );
  }

  #[test]
  fn rc_action() {
    assert_str_action(Rc::new(take(1)));
    assert_bytes_action(Rc::new(bytes::take(1)));
  }

  #[test]
  fn rc_dyn_action() {
    assert_str_action(
      Rc::new(take(1)) as Rc<dyn Action<Text = str, State = (), Heap = (), Value = ()>>
    );
    assert_bytes_action(
      Rc::new(bytes::take(1)) as Rc<dyn Action<Text = [u8], State = (), Heap = (), Value = ()>>
    );
  }
}
