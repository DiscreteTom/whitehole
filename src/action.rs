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
pub unsafe trait Action {
  /// See [`Output::value`].
  type Value;
  /// See [`Input::state`].
  type State;
  /// See [`Input::heap`].
  type Heap;

  /// Try to digest some bytes from the input, optionally change the state of the parsing,
  /// and yield a value.
  /// Return [`None`] to reject.
  fn exec(&self, input: Input<&mut Self::State, &mut Self::Heap>) -> Option<Output<Self::Value>>;
}

unsafe impl<T: Action + ?Sized> Action for &T {
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  fn exec(&self, input: Input<&mut Self::State, &mut Self::Heap>) -> Option<Output<Self::Value>> {
    (**self).exec(input)
  }
}

unsafe impl<T: Action + ?Sized> Action for &mut T {
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  fn exec(&self, input: Input<&mut Self::State, &mut Self::Heap>) -> Option<Output<Self::Value>> {
    (**self).exec(input)
  }
}

unsafe impl<T: Action + ?Sized> Action for Box<T> {
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  fn exec(&self, input: Input<&mut Self::State, &mut Self::Heap>) -> Option<Output<Self::Value>> {
    self.as_ref().exec(input)
  }
}

unsafe impl<T: Action + ?Sized> Action for Rc<T> {
  type Value = T::Value;
  type State = T::State;
  type Heap = T::Heap;

  fn exec(&self, input: Input<&mut Self::State, &mut Self::Heap>) -> Option<Output<Self::Value>> {
    self.as_ref().exec(input)
  }
}

/// Simplify [`Action`]'s signature with `impl`.
/// To use `dyn`, see [`A_dyn`](crate::A_dyn).
///
/// Here are the expanded forms:
/// ```
/// # use whitehole::{action::{Action, Input, Output}, A, combinator::wrap};
/// # #[derive(Default)]
/// # struct MyValue;
/// # struct MyHeap;
/// # struct MyState;
/// # struct T;
/// # unsafe impl Action for T {
/// #   type Value = ();
/// #   type State = ();
/// #   type Heap = ();
/// #   fn exec(&self, _: Input<&mut Self::State, &mut Self::Heap>) -> Option<Output<Self::Value>> {
/// #     None
/// #   }
/// # }
/// # macro_rules! assert_type_match {
/// #   ($t:ty => $expected:ty) => {{
/// #     fn receiver(_: $expected) {}
/// #     fn generator() -> $t {
/// #       wrap(|input| input.digest(1)).select(|_| Default::default())
/// #     }
/// #     receiver(generator());
/// #   }};
/// # }
/// # assert_type_match!(
/// A!()
/// => impl Action<Value = (), State = (), Heap = ()>
/// # );
///
/// # assert_type_match!(
/// A!(MyValue)
/// => impl Action<Value = MyValue, State = (), Heap = ()>
/// # );
///
/// # assert_type_match!(
/// A!(MyValue, MyState)
/// => impl Action<Value = MyValue, State = MyState, Heap = ()>
/// # );
///
/// # assert_type_match!(
/// A!(MyValue, MyState, MyHeap)
/// => impl Action<Value = MyValue, State = MyState, Heap = MyHeap>
/// # );
///
/// # assert_type_match!(
/// A!(@T)
/// => impl Action<Value = <T as Action>::Value, State = <T as Action>::State, Heap = <T as Action>::Heap>
/// # );
///
/// # assert_type_match!(
/// A!(MyValue, @T)
/// => impl Action<Value = MyValue, State = <T as Action>::State, Heap = <T as Action>::Heap>
/// # );
/// ```
#[macro_export]
macro_rules! A {
  () => {
    impl $crate::action::Action<Value = (), State = (), Heap = ()>
  };
  ($value:ty) => {
    impl $crate::action::Action<Value = $value, State = (), Heap = ()>
  };
  ($value:ty, $state:ty) => {
    impl $crate::action::Action<Value = $value, State = $state, Heap = ()>
  };
  ($value:ty, $state:ty, $heap:ty) => {
    impl $crate::action::Action<Value = $value, State = $state, Heap = $heap>
  };
  (@$from:ident) => {
    impl $crate::action::Action<Value = <$from as $crate::action::Action>::Value, State = <$from as $crate::action::Action>::State, Heap = <$from as $crate::action::Action>::Heap>
  };
  ($value:ty, @$from:ident) => {
    impl $crate::action::Action<Value = $value, State = <$from as $crate::action::Action>::State, Heap = <$from as $crate::action::Action>::Heap>
  };
}

/// Simplify [`Action`]'s signature with `dyn`.
/// To use `impl`, see [`A`].
///
/// Here are the expanded forms:
/// ```
/// # use whitehole::{action::{Action, Input, Output}, A_dyn, combinator::wrap};
/// # #[derive(Default)]
/// # struct MyValue;
/// # struct MyHeap;
/// # struct MyState;
/// # struct T;
/// # unsafe impl Action for T {
/// #   type Value = ();
/// #   type State = ();
/// #   type Heap = ();
/// #   fn exec(&self, _: Input<&mut Self::State, &mut Self::Heap>) -> Option<Output<Self::Value>> {
/// #     None
/// #   }
/// # }
/// # macro_rules! assert_type_match {
/// #   ($t:ty => $expected:ty) => {{
/// #     fn receiver(_: Box<$expected>) {}
/// #     fn generator() -> Box<$t> {
/// #       Box::new(wrap(|input| input.digest(1)).select(|_| Default::default()))
/// #     }
/// #     receiver(generator());
/// #   }};
/// # }
/// # assert_type_match!(
/// A_dyn!()
/// => dyn Action<Value = (), State = (), Heap = ()>
/// # );
///
/// # assert_type_match!(
/// A_dyn!(MyValue)
/// => dyn Action<Value = MyValue, State = (), Heap = ()>
/// # );
///
/// # assert_type_match!(
/// A_dyn!(MyValue, MyState)
/// => dyn Action<Value = MyValue, State = MyState, Heap = ()>
/// # );
///
/// # assert_type_match!(
/// A_dyn!(MyValue, MyState, MyHeap)
/// => dyn Action<Value = MyValue, State = MyState, Heap = MyHeap>
/// # );
///
/// # assert_type_match!(
/// A_dyn!(@T)
/// => dyn Action<Value = <T as Action>::Value, State = <T as Action>::State, Heap = <T as Action>::Heap>
/// # );
///
/// # assert_type_match!(
/// A_dyn!(MyValue, @T)
/// => dyn Action<Value = MyValue, State = <T as Action>::State, Heap = <T as Action>::Heap>
/// # );
/// ```
#[macro_export]
macro_rules! A_dyn {
  () => {
    dyn $crate::action::Action<Value = (), State = (), Heap = ()>
  };
  ($value:ty) => {
    dyn $crate::action::Action<Value = $value, State = (), Heap = ()>
  };
  ($value:ty, $state:ty) => {
    dyn $crate::action::Action<Value = $value, State = $state, Heap = ()>
  };
  ($value:ty, $state:ty, $heap:ty) => {
    dyn $crate::action::Action<Value = $value, State = $state, Heap = $heap>
  };
  (@$from:ident) => {
    dyn $crate::action::Action<Value = <$from as $crate::action::Action>::Value, State = <$from as $crate::action::Action>::State, Heap = <$from as $crate::action::Action>::Heap>
  };
  ($value:ty, @$from:ident) => {
    dyn $crate::action::Action<Value = $value, State = <$from as $crate::action::Action>::State, Heap = <$from as $crate::action::Action>::Heap>
  };
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{combinator::wrap, instant::Instant};

  fn helper(t: A!()) -> Option<Output<()>> {
    t.exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap())
  }

  #[test]
  fn action_ref() {
    assert!(helper(&wrap(|input| input.digest(1))).is_some());
    assert!(helper(&mut wrap(|input| input.digest(1))).is_some());
  }

  #[test]
  fn action_dyn_ref() {
    assert!(helper(&wrap(|input| input.digest(1)) as &A_dyn!()).is_some());
    assert!(helper(&mut wrap(|input| input.digest(1)) as &mut A_dyn!()).is_some());
  }

  #[test]
  fn boxed_action() {
    let output = helper(Box::new(wrap(|input| input.digest(1))));
    assert_eq!(
      output,
      Some(Output {
        value: (),
        digested: 1
      })
    );
  }

  #[test]
  fn boxed_dyn_action() {
    assert!(helper(Box::new(wrap(|input| input.digest(1))) as Box<A_dyn!()>).is_some());
  }

  #[test]
  fn rc_action() {
    let output = helper(Rc::new(wrap(|input| input.digest(1))));
    assert_eq!(
      output,
      Some(Output {
        value: (),
        digested: 1
      })
    );
  }

  #[test]
  fn rc_dyn_action() {
    assert!(helper(Rc::new(wrap(|input| input.digest(1))) as Rc<A_dyn!()>).is_some());
  }
}
