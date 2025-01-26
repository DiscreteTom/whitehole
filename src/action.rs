//! The basic building block of a parser.
//!
//! Each [`Action`] is a small piece of parsing logic that
//! digest some bytes from the input, optionally change the state of the parsing,
//! and yield a value.
//!
//! For most cases, you don't need to use [`Action`] directly.
//! See [`Combinator`](crate::combinator::Combinator) and
//! [`Parser`](crate::parser::Parser) for more high-level APIs.
//!
//! # Stateless
//!
//! [`Action`]s are stateless and immutable,
//! but they can access external states to change their behavior.
//! See [`Input::state`] and [`Input::heap`] for more information.
//!
//! States are centrally managed by the parser,
//! so it's easy to realize peeking and backtracking.
//!
//! # Consume the [`Input`]
//!
//! If not consuming the `Input`:
//! - With `&Input`: [`Input::state`] and [`Input::heap`] can't be mutated.
//! - With `&mut Input`: [`Action`]s may [`std::mem::swap`] the `Input` to break the outer state.
//!
//! So we decide to consume the `Input` in [`Action::exec`].
//! If you need to use `Input` for multiple times, see [`Input::reborrow`].

mod input;
mod output;

use std::rc::Rc;

pub use input::*;
pub use output::*;

/// The basic building block of a parser.
/// See the [module level documentation](crate::action) for more information.
/// # Safety
/// The [`Output`] of [`Action::exec`] should satisfy the requirement of [`Output::digested`].
pub unsafe trait Action<Text: ?Sized = str, State = (), Heap = ()> {
  /// See [`Output::value`].
  type Value;

  /// Try to digest some bytes from the input, optionally change the state of the parsing,
  /// and yield a value.
  /// Return [`None`] to reject.
  fn exec(&self, input: Input<&Text, &mut State, &mut Heap>) -> Option<Output<Self::Value>>;
}

unsafe impl<Text: ?Sized, State, Heap, T: Action<Text, State, Heap> + ?Sized>
  Action<Text, State, Heap> for &T
{
  type Value = T::Value;

  #[inline]
  fn exec(&self, input: Input<&Text, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    (**self).exec(input)
  }
}

unsafe impl<Text: ?Sized, State, Heap, T: Action<Text, State, Heap> + ?Sized>
  Action<Text, State, Heap> for Box<T>
{
  type Value = T::Value;

  #[inline]
  fn exec(&self, input: Input<&Text, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    self.as_ref().exec(input)
  }
}

unsafe impl<Text: ?Sized, State, Heap, T: Action<Text, State, Heap> + ?Sized>
  Action<Text, State, Heap> for Rc<T>
{
  type Value = T::Value;

  #[inline]
  fn exec(&self, input: Input<&Text, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    self.as_ref().exec(input)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    combinator::{bytes, wrap},
    instant::Instant,
  };

  fn helper(t: impl Action<Value = ()>) {
    assert!(t
      .exec(Input::new(Instant::new("123"), &mut (), &mut ()))
      .is_some());
  }
  fn helper_bytes(t: impl Action<[u8], Value = ()>) {
    assert!(t
      .exec(Input::new(Instant::new(b"123"), &mut (), &mut ()))
      .is_some());
  }

  #[test]
  fn action_ref() {
    helper(&wrap(|input| input.digest(1)));
    helper_bytes(&bytes::wrap(|input| input.digest(1)));
  }

  #[test]
  fn action_dyn_ref() {
    helper(&wrap(|input| input.digest(1)) as &dyn Action<Value = ()>);
    helper_bytes(&bytes::wrap(|input| input.digest(1)) as &dyn Action<[u8], Value = ()>);
  }

  #[test]
  fn boxed_action() {
    helper(Box::new(wrap(|input| input.digest(1))));
    helper_bytes(Box::new(bytes::wrap(|input| input.digest(1))));
  }

  #[test]
  fn boxed_dyn_action() {
    helper(Box::new(wrap(|input| input.digest(1))) as Box<dyn Action<Value = ()>>);
    helper_bytes(
      Box::new(bytes::wrap(|input| input.digest(1))) as Box<dyn Action<[u8], Value = ()>>
    );
  }

  #[test]
  fn rc_action() {
    helper(Rc::new(wrap(|input| input.digest(1))));
    helper_bytes(Rc::new(bytes::wrap(|input| input.digest(1))));
  }

  #[test]
  fn rc_dyn_action() {
    helper(Rc::new(wrap(|input| input.digest(1))) as Rc<dyn Action<Value = ()>>);
    helper_bytes(Rc::new(bytes::wrap(|input| input.digest(1))) as Rc<dyn Action<[u8], Value = ()>>);
  }
}
