use crate::{
  action::{Action, Input, Output},
  combinator::Combinator,
};
use core::fmt;
use std::{cell::OnceCell, rc::Rc};

/// Use `Box<dyn>` to prevent recursive/infinite type.
/// Use `OnceCell` to initialize this later.
/// Use `Rc` to make this clone-able.
type RecurInner<Text, Value, State, Heap> =
  Rc<OnceCell<Box<dyn Action<Text, State, Heap, Value = Value>>>>;

/// See [`recur`] and [`recur_unchecked`].
///
/// You can't construct this directly.
/// This is not [`Clone`] because you can only set the action implementor once.
/// This must be used to set the action implementor before the action is executed.
#[must_use = "This must be used to set the action implementor before the action is executed."]
pub struct RecurSetter<Text: ?Sized, Value, State, Heap> {
  inner: RecurInner<Text, Value, State, Heap>,
}

impl<Text: ?Sized, Value, State, Heap> RecurSetter<Text, Value, State, Heap> {
  /// Consume self, set the action implementor.
  #[inline]
  pub fn set(self, action: Box<dyn Action<Text, State, Heap, Value = Value>>) {
    // we can use `ok` here because the setter will be consumed after this call
    self.inner.set(action).ok();
  }

  /// Consume self, set the action implementor by boxing the provided action.
  #[inline]
  pub fn boxed(self, p: impl Action<Text, State, Heap, Value = Value> + 'static) {
    self.set(Box::new(p));
  }
}

/// See [`recur`].
pub struct Recur<Text: ?Sized, Value, State, Heap> {
  inner: RecurInner<Text, Value, State, Heap>,
}

impl<Text: ?Sized, Value, State, Heap> fmt::Debug for Recur<Text, Value, State, Heap> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Recur").finish()
  }
}

impl<Text: ?Sized, Value, State, Heap> Clone for Recur<Text, Value, State, Heap> {
  #[inline]
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

unsafe impl<Text: ?Sized, Value, State, Heap> Action<Text, State, Heap>
  for Recur<Text, Value, State, Heap>
{
  type Value = Value;

  #[inline]
  fn exec(&self, input: Input<&Text, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    self.inner.get().unwrap().exec(input)
  }
}

/// Create a recursive action. Return the getter and setter.
/// # Caveats
/// The setter must be used to set the action implementor before the action is executed.
/// Otherwise, the action will panic during execution.
///
/// You need to handle [left recursion](https://en.wikipedia.org/wiki/Left_recursion) by yourself.
/// # Examples
/// ```
/// # use whitehole::{combinator::{recur, eat}, action::{Action, Input}, instant::Instant};
/// // create a recursive action, get the getter and setter
/// let (value, value_setter) = recur();
///
/// // an array consists of zero or more values separated by commas, enclosed in square brackets.
/// // you can use the `value` before it is defined
/// let array = eat('[') + (value() * ..).sep(',') + ']';
///
/// // before execute the `value`, you must set the action implementor.
/// // a value is either an array or a character 'a'
/// value_setter.boxed(array | 'a');
///
/// // now you can execute the `value`.
/// // it can have recursive structures
/// assert!(value()
///   .exec(Input::new(Instant::new("a"), &mut (), &mut ()))
///   .is_some());
/// assert!(value()
///   .exec(Input::new(Instant::new("[]"), &mut (), &mut ()))
///   .is_some());
/// assert!(value()
///   .exec(Input::new(Instant::new("[a]"), &mut (), &mut ()))
///   .is_some());
/// assert!(value()
///   .exec(Input::new(Instant::new("[[]]"), &mut (), &mut ()))
///   .is_some());
/// assert!(value()
///   .exec(Input::new(Instant::new("[a,a]"), &mut (), &mut ()))
///   .is_some());
/// assert!(value()
///   .exec(Input::new(Instant::new("[[],[]]"), &mut (), &mut ()))
///   .is_some());
/// assert!(value()
///   .exec(Input::new(Instant::new("[[a],[]]"), &mut (), &mut ()))
///   .is_some());
/// ```
pub fn recur<Text: ?Sized, Value, State, Heap>() -> (
  impl Fn() -> Combinator<Recur<Text, Value, State, Heap>>,
  RecurSetter<Text, Value, State, Heap>,
) {
  let inner = Rc::new(OnceCell::new());
  let setter = RecurSetter {
    inner: inner.clone(),
  };
  let getter = move || {
    Combinator::new(Recur {
      inner: inner.clone(),
    })
  };
  (getter, setter)
}

/// See [`recur_unchecked`].
pub struct RecurUnchecked<Text: ?Sized, Value, State, Heap> {
  inner: RecurInner<Text, Value, State, Heap>,
}

impl<Text: ?Sized, Value, State, Heap> fmt::Debug for RecurUnchecked<Text, Value, State, Heap> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("RecurUnchecked").finish()
  }
}

impl<Text: ?Sized, Value, State, Heap> Clone for RecurUnchecked<Text, Value, State, Heap> {
  #[inline]
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

unsafe impl<Text: ?Sized, Value, State, Heap> Action<Text, State, Heap>
  for RecurUnchecked<Text, Value, State, Heap>
{
  type Value = Value;

  #[inline]
  fn exec(&self, input: Input<&Text, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    debug_assert!(self.inner.get().is_some());
    unsafe { self.inner.get().unwrap_unchecked() }.exec(input)
  }
}

/// Create a recursive action. Return the getter and setter.
/// # Safety
/// The setter must be used to set the action implementor before the action is executed.
/// This will be checked using [`debug_assert!`].
/// # Caveats
/// You need to handle [left recursion](https://en.wikipedia.org/wiki/Left_recursion) by yourself.
/// # Examples
/// ```
/// # use whitehole::{combinator::{recur_unchecked, eat}, action::{Action, Input}, instant::Instant};
/// // create a recursive action, get the getter and setter
/// let (value, value_setter) = unsafe { recur_unchecked() };
///
/// // an array consists of zero or more values separated by commas, enclosed in square brackets.
/// // you can use the `value` before it is defined
/// let array = eat('[') + (value() * ..).sep(',') + ']';
///
/// // before execute the `value`, you must set the action implementor.
/// // a value is either an array or a character 'a'
/// value_setter.boxed(array | 'a');
///
/// // now you can execute the `value`.
/// // it can have recursive structures
/// assert!(value()
///   .exec(Input::new(Instant::new("a"), &mut (), &mut ()))
///   .is_some());
/// assert!(value()
///   .exec(Input::new(Instant::new("[]"), &mut (), &mut ()))
///   .is_some());
/// assert!(value()
///   .exec(Input::new(Instant::new("[a]"), &mut (), &mut ()))
///   .is_some());
/// assert!(value()
///   .exec(Input::new(Instant::new("[[]]"), &mut (), &mut ()))
///   .is_some());
/// assert!(value()
///   .exec(Input::new(Instant::new("[a,a]"), &mut (), &mut ()))
///   .is_some());
/// assert!(value()
///   .exec(Input::new(Instant::new("[[],[]]"), &mut (), &mut ()))
///   .is_some());
/// assert!(value()
///   .exec(Input::new(Instant::new("[[a],[]]"), &mut (), &mut ()))
///   .is_some());
/// ```
pub unsafe fn recur_unchecked<Text: ?Sized, Value, State, Heap>() -> (
  impl Fn() -> Combinator<RecurUnchecked<Text, Value, State, Heap>>,
  RecurSetter<Text, Value, State, Heap>,
) {
  let inner = Rc::new(OnceCell::new());
  let setter = RecurSetter {
    inner: inner.clone(),
  };
  let getter = move || {
    Combinator::new(RecurUnchecked {
      inner: inner.clone(),
    })
  };
  (getter, setter)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{combinator::eat, instant::Instant};

  #[test]
  fn test_recur() {
    let (value, value_setter) = recur();
    let array = || eat('[') + (value() * ..).sep(',') + ']';
    value_setter.boxed(array() | 'a');

    assert!(value()
      .exec(Input::new(Instant::new("a"), &mut (), &mut ()))
      .is_some());
    assert!(value()
      .exec(Input::new(Instant::new("[]"), &mut (), &mut ()))
      .is_some());
    assert!(value()
      .exec(Input::new(Instant::new("[a]"), &mut (), &mut ()))
      .is_some());
    assert!(value()
      .exec(Input::new(Instant::new("[[]]"), &mut (), &mut ()))
      .is_some());
    assert!(value()
      .exec(Input::new(Instant::new("[a,a]"), &mut (), &mut ()))
      .is_some());
    assert!(value()
      .exec(Input::new(Instant::new("[[],[]]"), &mut (), &mut ()))
      .is_some());
    assert!(value()
      .exec(Input::new(Instant::new("[[a],[]]"), &mut (), &mut ()))
      .is_some());

    // make sure clone-able
    let _ = value().clone();

    assert_eq!(format!("{:?}", value().action), "Recur");
  }

  #[test]
  #[should_panic]
  fn test_recur_panic() {
    let (value, _) = recur::<_, (), _, _>();
    value().exec(Input::new(Instant::new("a"), &mut (), &mut ()));
  }

  #[test]
  fn test_recur_unchecked() {
    let (value, value_setter) = unsafe { recur_unchecked() };
    let array = || eat('[') + (value() * ..).sep(',') + ']';
    value_setter.boxed(array() | 'a');

    assert!(value()
      .exec(Input::new(Instant::new("a"), &mut (), &mut ()))
      .is_some());
    assert!(value()
      .exec(Input::new(Instant::new("[]"), &mut (), &mut ()))
      .is_some());
    assert!(value()
      .exec(Input::new(Instant::new("[a]"), &mut (), &mut ()))
      .is_some());
    assert!(value()
      .exec(Input::new(Instant::new("[[]]"), &mut (), &mut ()))
      .is_some());
    assert!(value()
      .exec(Input::new(Instant::new("[a,a]"), &mut (), &mut ()))
      .is_some());
    assert!(value()
      .exec(Input::new(Instant::new("[[],[]]"), &mut (), &mut ()))
      .is_some());
    assert!(value()
      .exec(Input::new(Instant::new("[[a],[]]"), &mut (), &mut ()))
      .is_some());

    // make sure clone-able
    let _ = value().clone();

    assert_eq!(format!("{:?}", value().action), "RecurUnchecked");
  }

  #[test]
  #[should_panic]
  fn test_recur_unchecked_panic() {
    let (value, _) = unsafe { recur_unchecked::<_, (), _, _>() };
    value().exec(Input::new(Instant::new("a"), &mut (), &mut ()));
  }
}
