use crate::{
  action::{Action, Input, Output},
  combinator::{provided::create_value_combinator, Combinator},
  instant::Instant,
};

create_value_combinator!(Till, "See [`till`].");

unsafe impl Action for Till<&str> {
  type Text = str;
  type State = ();
  type Heap = ();
  type Value = ();

  #[inline]
  fn exec(
    &self,
    input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    input.instant.rest().find(self.inner).map(|i| unsafe {
      input
        .instant
        .accept_unchecked(i.unchecked_add(self.inner.len()))
    })
  }
}

unsafe impl Action for Till<String> {
  type Text = str;
  type State = ();
  type Heap = ();
  type Value = ();

  #[inline]
  fn exec(
    &self,
    input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    input.instant.rest().find(&self.inner).map(|i| unsafe {
      input
        .instant
        .accept_unchecked(i.unchecked_add(self.inner.len()))
    })
  }
}

unsafe impl Action for Till<char> {
  type Text = str;
  type State = ();
  type Heap = ();
  type Value = ();

  #[inline]
  fn exec(
    &self,
    input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    input.instant.rest().find(self.inner).map(|i| unsafe {
      input
        .instant
        .accept_unchecked(i.unchecked_add(self.inner.len_utf8()))
    })
  }
}

unsafe impl Action for Till<()> {
  type Text = str;
  type State = ();
  type Heap = ();
  type Value = ();

  #[inline]
  fn exec(
    &self,
    input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    unsafe {
      input
        .instant
        .accept_unchecked(input.instant.rest().as_bytes().len())
    }
    .into()
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
  use crate::{action::Action, digest::Digest, instant::Instant};
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
  }
}
