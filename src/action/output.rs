use crate::{digest::Digest, instant::Instant};

/// The output of [`Action::exec`](crate::action::Action::exec).
/// Usually built by [`Instant::accept`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output<Value = ()> {
  /// The yielded value.
  pub value: Value,
  /// How many bytes are digested by this action.
  /// The value is validate by [`Digest::validate`].
  pub digested: usize,
}

impl<Text: ?Sized + Digest> Instant<&Text> {
  /// Try to build an [`Output`] by digesting `n` bytes.
  /// # Safety
  /// You should ensure that `n` is valid according to [`Digest::validate`].
  /// This will be checked using [`debug_assert!`].
  #[inline]
  pub unsafe fn accept_unchecked(&self, n: usize) -> Output<()> {
    debug_assert!(self.rest().validate(n));
    Output {
      value: (),
      digested: n,
    }
  }

  /// Try to build an [`Output`] by digesting `n` bytes.
  /// Return [`Some`] if `n` is valid according to [`Digest::validate`].
  #[inline]
  pub fn accept(&self, n: usize) -> Option<Output<()>> {
    self
      .rest()
      .validate(n)
      .then(|| unsafe { self.accept_unchecked(n) })
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

  /// Converts from `&Output<T>` to `Output<&T>`.
  #[inline]
  pub const fn as_ref(&self) -> Output<&Value> {
    Output {
      value: &self.value,
      digested: self.digested,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::instant::Instant;

  #[test]
  fn instant_accept_unchecked() {
    let instant = Instant::new("123");
    assert_eq!(unsafe { instant.accept_unchecked(0).digested }, 0);
    assert_eq!(unsafe { instant.accept_unchecked(1).digested }, 1);
    assert_eq!(unsafe { instant.accept_unchecked(2).digested }, 2);
    assert_eq!(unsafe { instant.accept_unchecked(3).digested }, 3);

    let instant = Instant::new(b"123" as &[u8]);
    assert_eq!(unsafe { instant.accept_unchecked(0).digested }, 0);
    assert_eq!(unsafe { instant.accept_unchecked(1).digested }, 1);
    assert_eq!(unsafe { instant.accept_unchecked(2).digested }, 2);
    assert_eq!(unsafe { instant.accept_unchecked(3).digested }, 3);
  }

  #[test]
  #[should_panic]
  fn instant_accept_unchecked_overflow() {
    let instant = Instant::new("123");
    unsafe { instant.accept_unchecked(4) };
  }

  #[test]
  #[should_panic]
  fn instant_bytes_accept_unchecked_overflow() {
    let instant = Instant::new(b"123" as &[u8]);
    unsafe { instant.accept_unchecked(4) };
  }

  #[test]
  #[should_panic]
  fn instant_accept_unchecked_invalid_code_point() {
    let instant = Instant::new("好");
    unsafe { instant.accept_unchecked(1) };
  }

  #[test]
  fn instant_accept() {
    let instant = Instant::new("123");
    assert_eq!(instant.accept(0).map(|output| output.digested), Some(0));
    assert_eq!(instant.accept(1).map(|output| output.digested), Some(1));
    assert_eq!(instant.accept(2).map(|output| output.digested), Some(2));
    assert_eq!(instant.accept(3).map(|output| output.digested), Some(3));
    assert!(instant.accept(4).is_none());

    let instant = Instant::new("好");
    assert_eq!(instant.accept(0).map(|output| output.digested), Some(0));
    assert!(instant.accept(1).is_none());
    assert!(instant.accept(2).is_none());
    assert_eq!(instant.accept(3).map(|output| output.digested), Some(3));
    assert!(instant.accept(4).is_none());

    let instant = Instant::new(b"123" as &[u8]);
    assert_eq!(instant.accept(0).map(|output| output.digested), Some(0));
    assert_eq!(instant.accept(1).map(|output| output.digested), Some(1));
    assert_eq!(instant.accept(2).map(|output| output.digested), Some(2));
    assert_eq!(instant.accept(3).map(|output| output.digested), Some(3));
    assert!(instant.accept(4).is_none());
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

  #[test]
  fn output_as_ref() {
    let o = Output {
      digested: 1,
      value: 1,
    };
    assert_eq!(o.as_ref().digested, 1);
    assert_eq!(o.as_ref().value, &1);
  }
}
