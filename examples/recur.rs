use std::{cell::OnceCell, rc::Rc};
use whitehole::{
  action::{Action, Input, Output},
  combinator::{eat, next, Combinator},
  parser::Builder,
};

// TODO: comments

// Use `Rc` to make it clone-able, use `OnceCell` to initialize it later,
// use `Box<dyn>` to prevent recursive/infinite type.
type RecurInner<Value, State, Heap> = Rc<OnceCell<Box<dyn Action<State, Heap, Value = Value>>>>;

/// See [`recur`] and [`recur_unchecked`].
///
/// You can't construct this directly.
/// This is not [`Clone`] because you can only set the parser once.
/// This must be used to set the parser.
#[must_use = "Setter must be used to set the parser"]
pub struct RecurSetter<Value, State, Heap> {
  inner: RecurInner<Value, State, Heap>,
}

impl<Value, State, Heap> RecurSetter<Value, State, Heap> {
  /// Consume self, set the parser.
  #[inline]
  pub fn set(self, parser: Box<dyn Action<State, Heap, Value = Value>>) {
    // we can use `ok` here because the setter will be dropped after this call
    self.inner.set(parser).ok();
  }

  /// Consume self, set the parser by boxing the parser.
  #[inline]
  pub fn boxed(self, p: impl Action<State, Heap, Value = Value> + 'static) {
    self.set(Box::new(p));
  }
}

/// See [`recur`].
pub struct Recur<Value, State, Heap> {
  rc: RecurInner<Value, State, Heap>,
}

unsafe impl<Value, State, Heap> Action<State, Heap> for Recur<Value, State, Heap> {
  type Value = Value;

  fn exec(&self, input: Input<&str, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    self.rc.get().unwrap().exec(input)
  }
}

/// # Caveats
/// The setter must be used to set the parser before calling `getter().parse`.
pub fn recur<Value, State, Heap>() -> (
  impl Fn() -> Combinator<Recur<Value, State, Heap>>,
  RecurSetter<Value, State, Heap>,
) {
  let setter = RecurSetter {
    inner: Rc::new(OnceCell::new()),
  };
  let getter = {
    let rc = setter.inner.clone();
    move || Combinator::new(Recur { rc: rc.clone() })
  };
  (getter, setter)
}

/// See [`recur_unchecked`].
pub struct RecurUnchecked<Value, State, Heap> {
  rc: RecurInner<Value, State, Heap>,
}

unsafe impl<Value, State, Heap> Action<State, Heap> for RecurUnchecked<Value, State, Heap> {
  type Value = Value;

  fn exec(&self, input: Input<&str, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    unsafe { self.rc.get().unwrap_unchecked() }.exec(input)
  }
}

/// # Safety
/// The setter must be used to set the parser before calling `getter().parse`.
pub unsafe fn recur_unchecked<Value, State, Heap>() -> (
  impl Fn() -> Combinator<RecurUnchecked<Value, State, Heap>>,
  RecurSetter<Value, State, Heap>,
) {
  let setter = RecurSetter {
    inner: Rc::new(OnceCell::new()),
  };
  let getter = {
    let rc = setter.inner.clone();
    move || Combinator::new(RecurUnchecked { rc: rc.clone() })
  };
  (getter, setter)
}

fn main() {
  let mut parser = Builder::new()
    .entry({
      let number = next(|c| c.is_ascii_digit()) * (1..);
      let (exp, exp_setter) = unsafe { recur_unchecked() };
      exp_setter.boxed(number | (eat('-') + exp()));

      exp()
    })
    .build("----123");

  while let Some(node) = parser.parse() {
    println!("{:?}", node);
  }

  let rest = parser.instant().rest();
  if rest.is_empty() {
    println!("Parsing successful!");
  } else {
    println!("Parsing failed!");
    println!(
      "Parsing failed, remaining: {:?}",
      &rest[..100.min(rest.len())]
    );
  }
}
