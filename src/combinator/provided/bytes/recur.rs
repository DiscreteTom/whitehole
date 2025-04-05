use crate::{
  action::{Action, Input, Output},
  combinator::{provided::create_recur, Combinator},
  instant::Instant,
};
use core::fmt;
use std::{cell::OnceCell, rc::Rc};

create_recur!([u8]);

/// Create a recursive action. Return the getter and setter.
/// # Caveats
/// The setter must be used to set the action implementor before the action is executed.
/// Otherwise, the action will panic during execution.
///
/// You need to handle [left recursion](https://en.wikipedia.org/wiki/Left_recursion) by yourself.
/// # Examples
/// ```
/// # use whitehole::{combinator::bytes, parser::Parser};
/// // create a recursive action, get the getter and setter
/// let (value, setter) = bytes::recur();
///
/// // an array consists of zero or more values separated by commas, enclosed in square brackets.
/// // you can use the `value` before it is defined
/// let array = bytes::eat(b'[') + (value() * ..).sep(b',') + b']';
///
/// // before executing `value`, you must set the action implementor.
/// // a value is either an array or a character 'a'
/// setter.boxed(array | b'a');
///
/// // now you can execute the `value`.
/// // it can have recursive structures
/// assert_eq!(Parser::builder().entry(value()).build(b"a").next().unwrap().digested, 1);
/// assert_eq!(Parser::builder().entry(value()).build(b"[]").next().unwrap().digested, 2);
/// assert_eq!(Parser::builder().entry(value()).build(b"[a]").next().unwrap().digested, 3);
/// assert_eq!(Parser::builder().entry(value()).build(b"[[]]").next().unwrap().digested, 4);
/// assert_eq!(Parser::builder().entry(value()).build(b"[a,a]").next().unwrap().digested, 5);
/// assert_eq!(Parser::builder().entry(value()).build(b"[[],[]]").next().unwrap().digested, 7);
/// assert_eq!(Parser::builder().entry(value()).build(b"[[a],[]]").next().unwrap().digested, 8);
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
/// # use whitehole::{combinator::bytes, parser::Parser};
/// // create a recursive action, get the getter and setter
/// let (value, setter) = unsafe { bytes::recur_unchecked() };
///
/// // an array consists of zero or more values separated by commas, enclosed in square brackets.
/// // you can use the `value` before it is defined
/// let array = bytes::eat(b'[') + (value() * ..).sep(b',') + b']';
///
/// // before executing the `value`, you must set the action implementor.
/// // a value is either an array or a character 'a'
/// setter.boxed(array | b'a');
///
/// // now you can execute the `value`.
/// // it can have recursive structures
/// assert_eq!(Parser::builder().entry(value()).build(b"a").next().unwrap().digested, 1);
/// assert_eq!(Parser::builder().entry(value()).build(b"[]").next().unwrap().digested, 2);
/// assert_eq!(Parser::builder().entry(value()).build(b"[a]").next().unwrap().digested, 3);
/// assert_eq!(Parser::builder().entry(value()).build(b"[[]]").next().unwrap().digested, 4);
/// assert_eq!(Parser::builder().entry(value()).build(b"[a,a]").next().unwrap().digested, 5);
/// assert_eq!(Parser::builder().entry(value()).build(b"[[],[]]").next().unwrap().digested, 7);
/// assert_eq!(Parser::builder().entry(value()).build(b"[[a],[]]").next().unwrap().digested, 8);
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
  use crate::{combinator::bytes::eat, digest::Digest, instant::Instant};
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
    let array = || eat(b'[') + (value() * ..).sep(b',') + b']';
    value_setter.boxed(array() | b'a');

    helper(value(), b"a", Some(1));
    helper(value(), b"[]", Some(2));
    helper(value(), b"[a]", Some(3));
    helper(value(), b"[[]]", Some(4));
    helper(value(), b"[a,a]", Some(5));
    helper(value(), b"[[],[]]", Some(7));
    helper(value(), b"[[a],[]]", Some(8));

    // make sure clone-able
    let _ = value().clone();

    assert_eq!(format!("{:?}", value().action), "Recur");
  }

  #[test]
  #[should_panic]
  fn test_recur_panic() {
    let (value, _) = recur::<()>();
    value().exec(Input {
      instant: &Instant::new(b"a"),
      state: &mut (),
      heap: &mut (),
    });
  }

  #[test]
  fn test_recur_unchecked() {
    let (value, value_setter) = unsafe { recur_unchecked() };
    let array = || eat(b'[') + (value() * ..).sep(b',') + b']';
    value_setter.boxed(array() | b'a');

    helper(value(), b"a", Some(1));
    helper(value(), b"[]", Some(2));
    helper(value(), b"[a]", Some(3));
    helper(value(), b"[[]]", Some(4));
    helper(value(), b"[a,a]", Some(5));
    helper(value(), b"[[],[]]", Some(7));
    helper(value(), b"[[a],[]]", Some(8));

    // make sure clone-able
    let _ = value().clone();

    assert_eq!(format!("{:?}", value().action), "RecurUnchecked");
  }

  #[test]
  #[should_panic]
  fn test_recur_unchecked_panic() {
    let (value, _) = unsafe { recur_unchecked::<()>() };
    value().exec(Input {
      instant: &Instant::new(b"a"),
      state: &mut (),
      heap: &mut (),
    });
  }
}
