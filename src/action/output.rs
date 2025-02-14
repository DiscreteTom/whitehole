use super::Input;
use crate::digest::Digest;

/// The output of [`Action::exec`](crate::action::Action::exec).
/// Usually built by [`Input::digest`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output<Value = ()> {
  /// The yielded value.
  pub value: Value,
  /// How many bytes are digested by this action.
  /// The value is validate by [`Digest::validate`].
  pub digested: usize,
}

impl<'a, Text: ?Sized, StateRef, HeapRef> Input<&'a Text, StateRef, HeapRef>
where
  &'a Text: Digest,
{
  /// Validate if it is ok to digest `n` bytes.
  /// See [`Digest::validate`] for more information.
  #[inline]
  pub fn validate(&self, n: usize) -> bool {
    self.instant().rest().validate(n)
  }

  /// Try to build an [`Output`] by digesting `n` bytes.
  /// # Safety
  /// You should ensure that `n` is valid according to [`Self::validate`].
  /// This will be checked using [`debug_assert!`].
  #[inline]
  pub unsafe fn digest_unchecked(&self, n: usize) -> Output<()> {
    debug_assert!(self.validate(n));
    Output {
      value: (),
      digested: n,
    }
  }

  /// Try to build an [`Output`] by digesting `n` bytes.
  /// Return [`Some`] if `n` is valid according to [`Self::validate`].
  #[inline]
  pub fn digest(&self, n: usize) -> Option<Output<()>> {
    self
      .validate(n)
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
  fn input_validate() {
    let mut state = ();
    let mut heap = ();

    let input = Input::new(Instant::new("123"), &mut state, &mut heap);
    assert!(input.validate(0));
    assert!(input.validate(1));
    assert!(input.validate(2));
    assert!(input.validate(3));
    assert!(!input.validate(4));

    let input = Input::new(Instant::new("好"), &mut state, &mut heap);
    assert!(input.validate(0));
    assert!(!input.validate(1));
    assert!(!input.validate(2));
    assert!(input.validate(3));

    let input = Input::new(Instant::new(b"123" as &[u8]), &mut state, &mut heap);
    assert!(input.validate(0));
    assert!(input.validate(1));
    assert!(input.validate(2));
    assert!(input.validate(3));
    assert!(!input.validate(4));
  }

  #[test]
  fn input_digest_unchecked() {
    let mut state = ();
    let mut heap = ();

    let input = Input::new(Instant::new("123"), &mut state, &mut heap);
    assert_eq!(unsafe { input.digest_unchecked(0).digested }, 0);
    assert_eq!(unsafe { input.digest_unchecked(1).digested }, 1);
    assert_eq!(unsafe { input.digest_unchecked(2).digested }, 2);
    assert_eq!(unsafe { input.digest_unchecked(3).digested }, 3);

    let input = Input::new(Instant::new(b"123" as &[u8]), &mut state, &mut heap);
    assert_eq!(unsafe { input.digest_unchecked(0).digested }, 0);
    assert_eq!(unsafe { input.digest_unchecked(1).digested }, 1);
    assert_eq!(unsafe { input.digest_unchecked(2).digested }, 2);
    assert_eq!(unsafe { input.digest_unchecked(3).digested }, 3);
  }

  #[test]
  #[should_panic]
  fn input_digest_unchecked_overflow() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new(Instant::new("123"), &mut state, &mut heap);
    unsafe { input.digest_unchecked(4) };
  }

  #[test]
  #[should_panic]
  fn input_bytes_digest_unchecked_overflow() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new(Instant::new(b"123" as &[u8]), &mut state, &mut heap);
    unsafe { input.digest_unchecked(4) };
  }

  #[test]
  #[should_panic]
  fn input_digest_unchecked_invalid_code_point() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new(Instant::new("好"), &mut state, &mut heap);
    unsafe { input.digest_unchecked(1) };
  }

  #[test]
  fn input_digest() {
    let mut state = ();
    let mut heap = ();

    let input = Input::new(Instant::new("123"), &mut state, &mut heap);
    assert_eq!(input.digest(0).map(|output| output.digested), Some(0));
    assert_eq!(input.digest(1).map(|output| output.digested), Some(1));
    assert_eq!(input.digest(2).map(|output| output.digested), Some(2));
    assert_eq!(input.digest(3).map(|output| output.digested), Some(3));
    assert!(input.digest(4).is_none());

    let input = Input::new(Instant::new("好"), &mut state, &mut heap);
    assert_eq!(input.digest(0).map(|output| output.digested), Some(0));
    assert!(input.digest(1).is_none());
    assert!(input.digest(2).is_none());
    assert_eq!(input.digest(3).map(|output| output.digested), Some(3));
    assert!(input.digest(4).is_none());

    let input = Input::new(Instant::new(b"123" as &[u8]), &mut state, &mut heap);
    assert_eq!(input.digest(0).map(|output| output.digested), Some(0));
    assert_eq!(input.digest(1).map(|output| output.digested), Some(1));
    assert_eq!(input.digest(2).map(|output| output.digested), Some(2));
    assert_eq!(input.digest(3).map(|output| output.digested), Some(3));
    assert!(input.digest(4).is_none());
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
