use crate::{
  action::{Action, Context, Output},
  combinator::{create_value_combinator, Combinator},
  digest::Digest,
  instant::Instant,
};

create_value_combinator!(Till, "See [`till`].");

unsafe impl<State, Heap> Action<str, State, Heap> for Till<&str> {
  type Value = ();

  #[inline]
  fn exec(&self, instant: &Instant<&str>, _: Context<&mut State, &mut Heap>) -> Option<Output<()>> {
    instant
      .rest()
      .find(self.inner)
      .map(|i| unsafe { instant.accept_unchecked(i.unchecked_add(self.inner.len())) })
  }
}

unsafe impl<State, Heap> Action<str, State, Heap> for Till<String> {
  type Value = ();

  #[inline]
  fn exec(&self, instant: &Instant<&str>, _: Context<&mut State, &mut Heap>) -> Option<Output<()>> {
    instant
      .rest()
      .find(&self.inner)
      .map(|i| unsafe { instant.accept_unchecked(i.unchecked_add(self.inner.len())) })
  }
}

unsafe impl<State, Heap> Action<str, State, Heap> for Till<char> {
  type Value = ();

  #[inline]
  fn exec(&self, instant: &Instant<&str>, _: Context<&mut State, &mut Heap>) -> Option<Output<()>> {
    instant
      .rest()
      .find(self.inner)
      .map(|i| unsafe { instant.accept_unchecked(i.unchecked_add(self.inner.len_utf8())) })
  }
}

unsafe impl<State, Heap> Action<[u8], State, Heap> for Till<u8> {
  type Value = ();

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&[u8]>,
    _: Context<&mut State, &mut Heap>,
  ) -> Option<Output<()>> {
    instant
      .rest()
      .iter()
      .enumerate()
      .find(|(_, b)| **b == self.inner)
      .map(|(i, _)| unsafe { instant.accept_unchecked(i.unchecked_add(1)) })
  }
}

unsafe impl<State, Heap> Action<[u8], State, Heap> for Till<&[u8]> {
  type Value = ();

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&[u8]>,
    _: Context<&mut State, &mut Heap>,
  ) -> Option<Output<()>> {
    // TODO: optimize
    if !self.inner.is_empty() {
      instant
        .rest()
        .windows(self.inner.len())
        .enumerate()
        .find(|(_, window)| *window == self.inner)
        .map(|(i, _)| unsafe { instant.accept_unchecked(i.unchecked_add(self.inner.len())) })
    } else {
      // window length can't be zero so we need special handling
      Some(Output {
        digested: 0,
        value: (),
      })
    }
  }
}

unsafe impl<const N: usize, State, Heap> Action<[u8], State, Heap> for Till<&[u8; N]> {
  type Value = ();

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&[u8]>,
    _: Context<&mut State, &mut Heap>,
  ) -> Option<Output<()>> {
    // TODO: optimize
    if N != 0 {
      instant
        .rest()
        .windows(N)
        .enumerate()
        .find(|(_, window)| *window == self.inner)
        .map(|(i, _)| unsafe { instant.accept_unchecked(i.unchecked_add(N)) })
    } else {
      // window length can't be zero so we need special handling
      Some(Output {
        digested: 0,
        value: (),
      })
    }
  }
}

unsafe impl<State, Heap> Action<[u8], State, Heap> for Till<Vec<u8>> {
  type Value = ();

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&[u8]>,
    _: Context<&mut State, &mut Heap>,
  ) -> Option<Output<()>> {
    // TODO: optimize
    if !self.inner.is_empty() {
      instant
        .rest()
        .windows(self.inner.len())
        .enumerate()
        .find(|(_, window)| *window == self.inner)
        .map(|(i, _)| unsafe { instant.accept_unchecked(i.unchecked_add(self.inner.len())) })
    } else {
      // window length can't be zero so we need special handling
      Some(Output {
        digested: 0,
        value: (),
      })
    }
  }
}

unsafe impl<Text: ?Sized + Digest, State, Heap> Action<Text, State, Heap> for Till<()> {
  type Value = ();

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    _: Context<&mut State, &mut Heap>,
  ) -> Option<Output<()>> {
    unsafe { instant.accept_unchecked(instant.rest().as_bytes().len()) }.into()
  }
}

/// Return a combinator to match the provided pattern, eat all the bytes
/// to the end of the first occurrence of the pattern (inclusive).
/// # Caveats
/// Empty patterns are allowed and will always accept 0 bytes,
/// even when [`Instant::rest`] is empty.
/// `()` will accept 0 bytes when [`Instant::rest`] is empty.
/// Be careful with infinite loops.
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
/// till(()) // with (), eat till the end
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
/// till(()) // with (), eat till the end
/// # );
/// ```
#[inline]
pub const fn till<T>(pattern: T) -> Combinator<Till<T>> {
  Combinator::new(Till::new(pattern))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{action::Action, instant::Instant};
  use std::{ops::RangeFrom, slice::SliceIndex};

  fn helper<Text: ?Sized + Digest>(
    action: impl Action<Text, Value = ()>,
    input: &Text,
    digested: Option<usize>,
  ) where
    RangeFrom<usize>: SliceIndex<Text, Output = Text>,
  {
    assert_eq!(
      action
        .exec(
          &Instant::new(input),
          Context {
            state: &mut (),
            heap: &mut ()
          }
        )
        .map(|o| o.digested),
      digested
    )
  }

  #[test]
  fn test_till() {
    // char
    helper(till(';'), "123;456", Some(4));
    helper(till(';'), "123456", None);

    // &str
    helper(till("end"), "123end456", Some(6));
    helper(till("end"), "123456", None);
    helper(till(""), "123456", Some(0));

    // String
    helper(till("end".to_string()), "123end456", Some(6));
    helper(till("end".to_string()), "123456", None);
    helper(till("".to_string()), "123456", Some(0));

    // ()
    helper(till(()), "123", Some(3));
    helper(till(()), "", Some(0));

    // u8
    helper(till(b';'), b"123;456", Some(4));
    helper(till(b';'), b"123456", None);

    // [u8, N]
    helper(till(b"end"), b"123end456", Some(6));
    helper(till(b"end"), b"123456", None);
    helper(till(b""), b"123456", Some(0));

    // &[u8]
    helper(till("end".to_string().as_bytes()), b"123end456", Some(6));
    helper(till("end".to_string().as_bytes()), b"123456", None);
    helper(till("".to_string().as_bytes()), b"123456", Some(0));

    // Vec<u8>
    helper(till(vec![b'1', b'2', b'3']), b"123456", Some(3));
    helper(till(vec![b'1', b'2', b'3']), b"456", None);
    helper(till(vec![]), b"456", Some(0));

    // ()
    helper(till(()), b"123" as &[u8], Some(3));
    helper(till(()), b"" as &[u8], Some(0));
  }
}
