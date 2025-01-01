use super::Input;

/// The output of [`Action::exec`](crate::action::Action::exec).
/// Usually built by [`Input::digest`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output<Value> {
  /// The yielded value.
  pub value: Value,
  /// How many bytes are digested by this action.
  ///
  /// This is guaranteed to be no greater than the length of `input.instant().rest()`
  /// and is always a valid UTF-8 boundary for the corresponding [`Input`].
  /// `0` is always a valid value.
  pub digested: usize,
}

impl<StateRef, HeapRef> Input<'_, StateRef, HeapRef> {
  /// Try to build an [`Output`] by digesting `n` bytes.
  /// # Safety
  /// You should ensure that `n` is a valid UTF-8 boundary.
  /// This will be checked using [`debug_assert!`].
  /// For the checked version, see [`Self::digest`].
  ///
  /// See [`Output::digested`] for more information.
  #[inline]
  pub unsafe fn digest_unchecked(&self, n: usize) -> Output<()> {
    debug_assert!(self.instant().rest().is_char_boundary(n));
    Output {
      value: (),
      digested: n,
    }
  }

  /// Try to build an [`Output`] by digesting `n` bytes.
  /// Return [`Some`] if `n` is a valid UTF-8 boundary.
  ///
  /// See [`Output::digested`] for more information.
  #[inline]
  pub fn digest(&self, n: usize) -> Option<Output<()>> {
    self
      .instant()
      .rest()
      .is_char_boundary(n)
      .then(|| unsafe { self.digest_unchecked(n) })
  }
}

impl<Value> Output<Value> {
  /// Convert [`Self::value`] to a new value.
  #[inline]
  pub fn map<NewValue>(self, f: impl FnOnce(Value) -> NewValue) -> Output<NewValue> {
    Output {
      value: f(self.value),
      digested: self.digested,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::instant::Instant;

  #[test]
  fn input_digest() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new(Instant::new("123"), &mut state, &mut heap).unwrap();
    assert_eq!(input.digest(3).map(|output| output.digested), Some(3));
    assert_eq!(input.digest(2).map(|output| output.digested), Some(2));
    assert_eq!(input.digest(1).map(|output| output.digested), Some(1));
    assert_eq!(input.digest(0).map(|output| output.digested), Some(0));
    assert!(input.digest(4).is_none());
  }

  #[test]
  fn input_digest_invalid_code_point() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new(Instant::new("好"), &mut state, &mut heap).unwrap();
    assert!(input.digest(1).is_none());
  }

  #[test]
  fn input_digest_unchecked() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new(Instant::new("123"), &mut state, &mut heap).unwrap();
    assert_eq!(unsafe { input.digest_unchecked(3).digested }, 3);
    assert_eq!(unsafe { input.digest_unchecked(2).digested }, 2);
    assert_eq!(unsafe { input.digest_unchecked(1).digested }, 1);
    assert_eq!(unsafe { input.digest_unchecked(0).digested }, 0);
  }

  #[test]
  #[should_panic]
  fn input_digest_unchecked_overflow() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new(Instant::new("123"), &mut state, &mut heap).unwrap();
    unsafe { input.digest_unchecked(4) };
  }

  #[test]
  #[should_panic]
  fn input_digest_unchecked_invalid_code_point() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new(Instant::new("好"), &mut state, &mut heap).unwrap();
    unsafe { input.digest_unchecked(1) };
  }

  #[test]
  fn output_map() {
    assert_eq!(
      Output {
        value: 1,
        digested: 0,
      }
      .map(|value| value + 1),
      Output {
        value: 2,
        digested: 0,
      }
    );
  }
}
