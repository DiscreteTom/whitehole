use std::{cell::OnceCell, rc::Rc};
use whitehole::{
  action::{Action, Input, Output},
  combinator::{eat, next, Combinator},
  parser::Builder,
};

// TODO: comments

// Use `Rc` to make it clone-able, use `OnceCell` to initialize it later,
// use `Box<dyn>` to prevent recursive/infinite type.
type RecurInner<Kind, State, Heap> =
  Rc<OnceCell<Box<dyn Action<Value = Kind, State = State, Heap = Heap>>>>;

/// See [`recur`] and [`recur_unchecked`].
///
/// You can't construct this directly.
/// This is not [`Clone`] because you can only set the parser once.
/// This must be used to set the parser.
#[must_use = "Setter must be used to set the parser"]
pub struct RecurSetter<Kind, State, Heap> {
  inner: RecurInner<Kind, State, Heap>,
}

impl<Kind, State, Heap> RecurSetter<Kind, State, Heap> {
  /// Consume self, set the parser.
  #[inline]
  pub fn set(self, parser: Box<dyn Action<Value = Kind, State = State, Heap = Heap>>) {
    // we can use `ok` here because the setter will be dropped after this call
    self.inner.set(parser).ok();
  }

  /// Consume self, set the parser by boxing the parser.
  #[inline]
  pub fn boxed(self, p: impl Action<Value = Kind, State = State, Heap = Heap> + 'static) {
    self.set(Box::new(p));
  }
}

/// See [`recur`].
pub struct Recur<Kind, State, Heap> {
  rc: RecurInner<Kind, State, Heap>,
}

unsafe impl<Kind, State, Heap> Action for Recur<Kind, State, Heap> {
  type Value = Kind;
  type State = State;
  type Heap = Heap;

  fn exec<'text>(&self, input: Input<'text, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    self.rc.get().unwrap().exec(input)
  }
}

/// # Caveats
/// The setter must be used to set the parser before calling `getter().parse`.
pub fn recur<Kind, State, Heap>() -> (
  impl Fn() -> Combinator<Recur<Kind, State, Heap>>,
  RecurSetter<Kind, State, Heap>,
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
pub struct RecurUnchecked<Kind, State, Heap> {
  rc: RecurInner<Kind, State, Heap>,
}

unsafe impl<Kind, State, Heap> Action for RecurUnchecked<Kind, State, Heap> {
  type Value = Kind;
  type State = State;
  type Heap = Heap;

  fn exec<'text>(&self, input: Input<'text, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    unsafe { self.rc.get().unwrap_unchecked() }.exec(input)
  }
}

/// # Safety
/// The setter must be used to set the parser before calling `getter().parse`.
pub unsafe fn recur_unchecked<Kind, State, Heap>() -> (
  impl Fn() -> Combinator<RecurUnchecked<Kind, State, Heap>>,
  RecurSetter<Kind, State, Heap>,
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
