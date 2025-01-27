use crate::{
  action::{Action, Input, Output},
  combinator::{create_value_combinator, Combinator},
  digest::Digest,
};

create_value_combinator!(Till, "See [`till`].");

unsafe impl<State, Heap> Action<str, State, Heap> for Till<&str> {
  type Value = ();

  #[inline]
  fn exec(&self, input: Input<&str, &mut State, &mut Heap>) -> Option<Output<()>> {
    input
      .instant()
      .rest()
      .find(self.inner)
      .map(|i| unsafe { input.digest_unchecked(i.unchecked_add(self.inner.len())) })
  }
}

unsafe impl<State, Heap> Action<str, State, Heap> for Till<String> {
  type Value = ();

  #[inline]
  fn exec(&self, input: Input<&str, &mut State, &mut Heap>) -> Option<Output<()>> {
    input
      .instant()
      .rest()
      .find(&self.inner)
      .map(|i| unsafe { input.digest_unchecked(i.unchecked_add(self.inner.len())) })
  }
}

unsafe impl<State, Heap> Action<str, State, Heap> for Till<char> {
  type Value = ();

  #[inline]
  fn exec(&self, input: Input<&str, &mut State, &mut Heap>) -> Option<Output<()>> {
    input
      .instant()
      .rest()
      .find(self.inner)
      .map(|i| unsafe { input.digest_unchecked(i.unchecked_add(self.inner.len_utf8())) })
  }
}

unsafe impl<State, Heap> Action<[u8], State, Heap> for Till<u8> {
  type Value = ();

  #[inline]
  fn exec(&self, input: Input<&[u8], &mut State, &mut Heap>) -> Option<Output<()>> {
    input
      .instant()
      .rest()
      .iter()
      .enumerate()
      .find(|(_, b)| **b == self.inner)
      .map(|(i, _)| unsafe { input.digest_unchecked(i.unchecked_add(1)) })
  }
}

unsafe impl<State, Heap> Action<[u8], State, Heap> for Till<&[u8]> {
  type Value = ();

  #[inline]
  fn exec(&self, input: Input<&[u8], &mut State, &mut Heap>) -> Option<Output<()>> {
    // TODO: optimize
    input
      .instant()
      .rest()
      .windows(self.inner.len())
      .enumerate()
      .find(|(_, window)| *window == self.inner)
      .map(|(i, _)| unsafe { input.digest_unchecked(i.unchecked_add(self.inner.len())) })
  }
}

unsafe impl<const N: usize, State, Heap> Action<[u8], State, Heap> for Till<&[u8; N]> {
  type Value = ();

  #[inline]
  fn exec(&self, input: Input<&[u8], &mut State, &mut Heap>) -> Option<Output<()>> {
    // TODO: optimize
    input
      .instant()
      .rest()
      .windows(N)
      .enumerate()
      .find(|(_, window)| *window == self.inner)
      .map(|(i, _)| unsafe { input.digest_unchecked(i.unchecked_add(N)) })
  }
}

unsafe impl<State, Heap> Action<[u8], State, Heap> for Till<Vec<u8>> {
  type Value = ();

  #[inline]
  fn exec(&self, input: Input<&[u8], &mut State, &mut Heap>) -> Option<Output<()>> {
    // TODO: optimize
    input
      .instant()
      .rest()
      .windows(self.inner.len())
      .enumerate()
      .find(|(_, window)| *window == self.inner)
      .map(|(i, _)| unsafe { input.digest_unchecked(i.unchecked_add(self.inner.len())) })
  }
}

unsafe impl<Text: ?Sized, State, Heap> Action<Text, State, Heap> for Till<()>
where
  for<'a> &'a Text: Digest,
{
  type Value = ();

  #[inline]
  fn exec(&self, input: Input<&Text, &mut State, &mut Heap>) -> Option<Output<()>> {
    unsafe { input.digest_unchecked(input.instant().rest().len()) }.into()
  }
}

/// Return a combinator to match the provided pattern, eat all the bytes
/// to the end of the first occurrence of the pattern (inclusive).
///
/// `""` (empty string) is allowed but be careful with infinite loops.
/// # Examples
/// For string (`&str`):
/// ```
/// # use whitehole::{combinator::{till, Combinator}, action::Action};
/// # fn t(_: Combinator<impl Action>) {}
/// # t(
/// till(';') // with char
/// # );
/// # t(
/// till("end") // with &str
/// # );
/// # t(
/// till("end".to_string()) // with String
/// # );
/// # t(
/// till(()) // with (), eat all rest
/// # );
/// ```
/// For bytes (`&[u8]`):
/// ```
/// # use whitehole::{combinator::{till, Combinator}, action::Action};
/// # fn t(_: Combinator<impl Action<[u8]>>) {}
/// # t(
/// till(b';') // with u8
/// # );
/// # t(
/// till(b"end") // with &[u8] or &[u8; N]
/// # );
/// # t(
/// till(vec![b'a']) // with Vec<u8>
/// # );
/// # t(
/// till(()) // with (), eat all rest
/// # );
/// ```
#[inline]
pub const fn till<T>(pattern: T) -> Combinator<Till<T>> {
  Combinator::new(Till::new(pattern))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    action::{Action, Input, Output},
    instant::Instant,
  };

  #[test]
  fn until_exec() {
    assert_eq!(
      till(';').exec(Input::new(Instant::new("123;456"), &mut (), &mut ())),
      Some(Output {
        value: (),
        digested: 4
      })
    );
    assert_eq!(
      till("end").exec(Input::new(Instant::new("123end456"), &mut (), &mut ())),
      Some(Output {
        value: (),
        digested: 6
      })
    );
    assert_eq!(
      till("end".to_string()).exec(Input::new(Instant::new("123end456"), &mut (), &mut ())),
      Some(Output {
        value: (),
        digested: 6
      })
    );
    assert_eq!(
      till(()).exec(Input::new(Instant::new("123"), &mut (), &mut ())),
      Some(Output {
        value: (),
        digested: 3
      })
    );
    assert_eq!(
      till(b';').exec(Input::new(Instant::new(b"123;456"), &mut (), &mut ())),
      Some(Output {
        value: (),
        digested: 4
      })
    );
    assert_eq!(
      till(b"end").exec(Input::new(Instant::new(b"123end456"), &mut (), &mut ())),
      Some(Output {
        value: (),
        digested: 6
      })
    );
    assert_eq!(
      till("end".to_string().as_bytes()).exec(Input::new(
        Instant::new(b"123end456"),
        &mut (),
        &mut ()
      )),
      Some(Output {
        value: (),
        digested: 6
      })
    );
    assert_eq!(
      till(vec![b'1', b'2', b'3']).exec(Input::new(Instant::new(b"123456"), &mut (), &mut ())),
      Some(Output {
        value: (),
        digested: 3
      })
    );
    assert_eq!(
      till(()).exec(Input::new(Instant::new(b"123" as &[u8]), &mut (), &mut ())),
      Some(Output {
        value: (),
        digested: 3
      })
    );
  }
}
