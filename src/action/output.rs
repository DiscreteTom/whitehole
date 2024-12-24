use super::Input;

/// The output of [`Action::exec`](crate::action::Action::exec).
/// Usually built by [`Input::digest`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output<'text, Value> {
  /// The yielded value.
  pub value: Value,
  /// The rest of the input text.
  pub rest: &'text str,
}

impl<'text, StateRef, HeapRef> Input<'text, StateRef, HeapRef> {
  /// Try to build an [`Output`] by digesting `n` bytes.
  /// Return [`None`] if the [`Output::rest`] can't be built
  /// as a valid UTF-8 string.
  #[inline]
  pub fn digest(&self, n: usize) -> Option<Output<'text, ()>> {
    self.rest().get(n..).map(|rest| Output { value: (), rest })
  }

  /// Try to build an [`Output`] by digesting `n` bytes.
  /// # Safety
  /// You should ensure that [`Output::rest`] can be built
  /// as a valid UTF-8 string.
  /// This will be checked using [`debug_assert!`].
  /// For the checked version, see [`Self::digest`].
  #[inline]
  pub unsafe fn digest_unchecked(&self, n: usize) -> Output<'text, ()> {
    debug_assert!(self.rest().get(n..).is_some());
    Output {
      value: (),
      rest: self.rest().get_unchecked(n..),
    }
  }
}

impl<'text, Value> Output<'text, Value> {
  /// Convert [`Self::value`] to a new value.
  #[inline]
  pub fn map<NewValue>(self, f: impl FnOnce(Value) -> NewValue) -> Output<'text, NewValue> {
    Output {
      value: f(self.value),
      rest: self.rest,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn input_digest() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new("123", 0, &mut state, &mut heap).unwrap();
    assert_eq!(input.digest(3).map(|output| output.rest), Some(""));
    assert_eq!(input.digest(2).map(|output| output.rest), Some("3"));
    assert_eq!(input.digest(1).map(|output| output.rest), Some("23"));
    assert_eq!(input.digest(0).map(|output| output.rest), Some("123"));
    assert!(input.digest(4).is_none());
  }

  #[test]
  fn input_digest_unchecked() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new("123", 0, &mut state, &mut heap).unwrap();
    assert_eq!(unsafe { input.digest_unchecked(3).rest }, "");
    assert_eq!(unsafe { input.digest_unchecked(2).rest }, "3");
    assert_eq!(unsafe { input.digest_unchecked(1).rest }, "23");
    assert_eq!(unsafe { input.digest_unchecked(0).rest }, "123");
  }

  #[test]
  #[should_panic]
  fn input_digest_unchecked_overflow() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new("123", 0, &mut state, &mut heap).unwrap();
    unsafe { input.digest_unchecked(4) };
  }

  #[test]
  #[should_panic]
  fn input_digest_unchecked_invalid_code_point() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new("好", 0, &mut state, &mut heap).unwrap();
    unsafe { input.digest_unchecked(1) };
  }

  #[test]
  fn output_map() {
    assert_eq!(
      Output {
        value: 1,
        rest: "123",
      }
      .map(|value| value + 1),
      Output {
        value: 2,
        rest: "123",
      }
    );
  }
}
