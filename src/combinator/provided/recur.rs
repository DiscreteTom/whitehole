use crate::{
  action::{Action, Input, Output},
  combinator::Combinator,
  instant::Instant,
};
use core::fmt;
use std::{cell::OnceCell, rc::Rc};

macro_rules! create_recur {
  ($text:ty) => {
    /// Use `Box<dyn>` to prevent recursive/infinite type.
    /// Use `OnceCell` to initialize this later.
    /// Use `Rc` to make this clone-able.
    pub type RecurInner<State, Heap, Value> =
      Rc<OnceCell<Box<dyn Action<Text = $text, State = State, Heap = Heap, Value = Value>>>>;

    /// See [`recur`] and [`recur_unchecked`].
    ///
    /// You can't construct this directly.
    /// This is not [`Clone`] because you can only set the action implementor once.
    /// This must be used to set the action implementor before the action is executed.
    #[must_use = "This must be used to set the action implementor before the action is executed."]
    pub struct RecurSetter<State = (), Heap = (), Value = ()> {
      inner: RecurInner<State, Heap, Value>,
    }

    impl<State, Heap, Value> RecurSetter<State, Heap, Value> {
      /// Create a new instance.
      #[inline]
      pub const fn new(inner: RecurInner<State, Heap, Value>) -> Self {
        Self { inner }
      }

      /// Consume self, set the action implementor.
      #[inline]
      pub fn set(
        self,
        action: Box<dyn Action<Text = $text, State = State, Heap = Heap, Value = Value>>,
      ) {
        // we can use `ok` here because the setter will be consumed after this call
        self.inner.set(action).ok();
      }

      /// Consume self, set the action implementor by boxing the provided action.
      #[inline]
      pub fn boxed(
        self,
        p: impl Action<Text = $text, State = State, Heap = Heap, Value = Value> + 'static,
      ) {
        self.set(Box::new(p));
      }
    }

    /// See [`recur`].
    pub struct Recur<State = (), Heap = (), Value = ()> {
      inner: RecurInner<State, Heap, Value>,
    }

    impl<State, Heap, Value> Recur<State, Heap, Value> {
      /// Create a new instance.
      #[inline]
      pub const fn new(inner: RecurInner<State, Heap, Value>) -> Self {
        Self { inner }
      }
    }

    impl<State, Heap, Value> fmt::Debug for Recur<State, Heap, Value> {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Recur").finish()
      }
    }

    impl<State, Heap, Value> Clone for Recur<State, Heap, Value> {
      #[inline]
      fn clone(&self) -> Self {
        Self {
          inner: self.inner.clone(),
        }
      }
    }

    unsafe impl<State, Heap, Value> Action for Recur<State, Heap, Value> {
      type Text = $text;
      type State = State;
      type Heap = Heap;
      type Value = Value;

      #[inline]
      fn exec(
        &self,
        input: Input<&Instant<&Self::Text>, &mut State, &mut Heap>,
      ) -> Option<Output<Self::Value>> {
        self.inner.get().unwrap().exec(input)
      }
    }

    /// See [`recur_unchecked`].
    pub struct RecurUnchecked<State = (), Heap = (), Value = ()> {
      inner: RecurInner<State, Heap, Value>,
    }

    impl<State, Heap, Value> RecurUnchecked<State, Heap, Value> {
      /// Create a new instance.
      #[inline]
      pub const fn new(inner: RecurInner<State, Heap, Value>) -> Self {
        Self { inner }
      }
    }

    impl<State, Heap, Value> fmt::Debug for RecurUnchecked<State, Heap, Value> {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RecurUnchecked").finish()
      }
    }

    impl<State, Heap, Value> Clone for RecurUnchecked<State, Heap, Value> {
      #[inline]
      fn clone(&self) -> Self {
        Self {
          inner: self.inner.clone(),
        }
      }
    }

    unsafe impl<State, Heap, Value> Action for RecurUnchecked<State, Heap, Value> {
      type Text = $text;
      type State = State;
      type Heap = Heap;
      type Value = Value;

      #[inline]
      fn exec(
        &self,
        input: Input<&Instant<&Self::Text>, &mut State, &mut Heap>,
      ) -> Option<Output<Self::Value>> {
        debug_assert!(self.inner.get().is_some());
        unsafe { self.inner.get().unwrap_unchecked() }.exec(input)
      }
    }
  };
}
pub(super) use create_recur;

create_recur!(str);

/// Create a recursive action. Return the getter and setter.
/// # Caveats
/// The setter must be used to set the action implementor before the action is executed.
/// Otherwise, the action will panic during execution.
///
/// You need to handle [left recursion](https://en.wikipedia.org/wiki/Left_recursion) by yourself.
/// # Examples
/// ```
/// # use whitehole::{combinator::{recur, eat}, parser::Parser};
/// // create a recursive action, get the getter and setter
/// let (value, setter) = recur();
///
/// // an array consists of zero or more values separated by commas, enclosed in square brackets.
/// // you can use the `value` before it is defined
/// let array = eat('[') + (value() * ..).sep(',') + ']';
///
/// // before executing `value`, you must set the action implementor.
/// // a value is either an array or a character 'a'
/// setter.boxed(array | 'a');
///
/// // now you can execute the `value`.
/// // it can have recursive structures
/// assert_eq!(Parser::builder().entry(value()).build("a").next().unwrap().digested, 1);
/// assert_eq!(Parser::builder().entry(value()).build("[]").next().unwrap().digested, 2);
/// assert_eq!(Parser::builder().entry(value()).build("[a]").next().unwrap().digested, 3);
/// assert_eq!(Parser::builder().entry(value()).build("[[]]").next().unwrap().digested, 4);
/// assert_eq!(Parser::builder().entry(value()).build("[a,a]").next().unwrap().digested, 5);
/// assert_eq!(Parser::builder().entry(value()).build("[[],[]]").next().unwrap().digested, 7);
/// assert_eq!(Parser::builder().entry(value()).build("[[a],[]]").next().unwrap().digested, 8);
/// ```
#[allow(clippy::type_complexity)]
pub fn recur<Value>() -> (
  impl Fn() -> Combinator<Recur<(), (), Value>>,
  RecurSetter<(), (), Value>,
) {
  let inner = Rc::new(OnceCell::new());
  let setter = RecurSetter::new(inner.clone());
  let getter = move || Combinator::new(Recur::new(inner.clone()));
  (getter, setter)
}

/// Create a recursive action. Return the getter and setter.
/// # Safety
/// The setter must be used to set the action implementor before the action is executed.
/// This will be checked using [`debug_assert!`].
/// # Caveats
/// You need to handle [left recursion](https://en.wikipedia.org/wiki/Left_recursion) by yourself.
/// # Examples
/// ```
/// # use whitehole::{combinator::{recur_unchecked, eat}, parser::Parser};
/// // create a recursive action, get the getter and setter
/// let (value, setter) = unsafe { recur_unchecked() };
///
/// // an array consists of zero or more values separated by commas, enclosed in square brackets.
/// // you can use the `value` before it is defined
/// let array = eat('[') + (value() * ..).sep(',') + ']';
///
/// // before executing the `value`, you must set the action implementor.
/// // a value is either an array or a character 'a'
/// setter.boxed(array | 'a');
///
/// // now you can execute the `value`.
/// // it can have recursive structures
/// assert_eq!(Parser::builder().entry(value()).build("a").next().unwrap().digested, 1);
/// assert_eq!(Parser::builder().entry(value()).build("[]").next().unwrap().digested, 2);
/// assert_eq!(Parser::builder().entry(value()).build("[a]").next().unwrap().digested, 3);
/// assert_eq!(Parser::builder().entry(value()).build("[[]]").next().unwrap().digested, 4);
/// assert_eq!(Parser::builder().entry(value()).build("[a,a]").next().unwrap().digested, 5);
/// assert_eq!(Parser::builder().entry(value()).build("[[],[]]").next().unwrap().digested, 7);
/// assert_eq!(Parser::builder().entry(value()).build("[[a],[]]").next().unwrap().digested, 8);
/// ```
#[allow(clippy::type_complexity)]
pub unsafe fn recur_unchecked<Value>() -> (
  impl Fn() -> Combinator<RecurUnchecked<(), (), Value>>,
  RecurSetter<(), (), Value>,
) {
  let inner = Rc::new(OnceCell::new());
  let setter = RecurSetter::new(inner.clone());
  let getter = move || Combinator::new(RecurUnchecked::new(inner.clone()));
  (getter, setter)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{combinator::eat, digest::Digest, instant::Instant};
  use std::{ops::RangeFrom, slice::SliceIndex};

  fn helper<Text: ?Sized + Digest>(
    action: impl Action<Text = Text, State = (), Heap = (), Value = ()>,
    input: &Text,
    digested: Option<usize>,
  ) where
    RangeFrom<usize>: SliceIndex<Text, Output = Text>,
  {
    assert_eq!(
      action
        .exec(Input {
          instant: &Instant::new(input),
          state: &mut (),
          heap: &mut ()
        })
        .map(|o| o.digested),
      digested
    )
  }

  #[test]
  fn test_recur() {
    let (value, value_setter) = recur();
    let array = || eat('[') + (value() * ..).sep(',') + ']';
    value_setter.boxed(array() | 'a');

    helper(value(), "a", Some(1));
    helper(value(), "[]", Some(2));
    helper(value(), "[a]", Some(3));
    helper(value(), "[[]]", Some(4));
    helper(value(), "[a,a]", Some(5));
    helper(value(), "[[],[]]", Some(7));
    helper(value(), "[[a],[]]", Some(8));

    // make sure clone-able
    let _ = value().clone();

    assert_eq!(format!("{:?}", value().action), "Recur");
  }

  #[test]
  #[should_panic]
  fn test_recur_panic() {
    let (value, _) = recur::<()>();
    value().exec(Input {
      instant: &Instant::new("a"),
      state: &mut (),
      heap: &mut (),
    });
  }

  #[test]
  fn test_recur_unchecked() {
    let (value, value_setter) = unsafe { recur_unchecked() };
    let array = || eat('[') + (value() * ..).sep(',') + ']';
    value_setter.boxed(array() | 'a');

    helper(value(), "a", Some(1));
    helper(value(), "[]", Some(2));
    helper(value(), "[a]", Some(3));
    helper(value(), "[[]]", Some(4));
    helper(value(), "[a,a]", Some(5));
    helper(value(), "[[],[]]", Some(7));
    helper(value(), "[[a],[]]", Some(8));

    // make sure clone-able
    let _ = value().clone();

    assert_eq!(format!("{:?}", value().action), "RecurUnchecked");
  }

  #[test]
  #[should_panic]
  fn test_recur_unchecked_panic() {
    let (value, _) = unsafe { recur_unchecked::<()>() };
    value().exec(Input {
      instant: &Instant::new("a"),
      state: &mut (),
      heap: &mut (),
    });
  }
}
