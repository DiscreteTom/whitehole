use crate::{
  action::{Action, Input, Output},
  combinator::Combinator,
};

#[derive(Copy, Clone, Debug)]
pub struct Take {
  n: usize,
}

impl Take {
  #[inline]
  const fn new(n: usize) -> Self {
    Self { n }
  }
}

unsafe impl<State, Heap> Action<str, State, Heap> for Take {
  type Value = ();

  #[inline]
  fn exec(&self, input: Input<&str, &mut State, &mut Heap>) -> Option<Output<()>> {
    let mut digested: usize = 0;
    let mut count: usize = 0;
    let mut chars = input.instant().rest().chars();
    while count < self.n {
      // no enough chars, try to digest more
      if let Some(c) = chars.next() {
        digested = unsafe { digested.unchecked_add(c.len_utf8()) };
        // SAFETY: count is always smaller than self which is a usize
        count = unsafe { count.unchecked_add(1) };
      } else {
        // no enough chars, reject
        return None;
      }
    }
    // enough chars
    unsafe { input.digest_unchecked(digested) }.into()
  }
}

unsafe impl<State, Heap> Action<[u8], State, Heap> for Take {
  type Value = ();

  #[inline]
  fn exec(&self, input: Input<&[u8], &mut State, &mut Heap>) -> Option<Output<()>> {
    input.digest(self.n)
  }
}

/// Returns a combinator to take the next `n` undigested [`char`]s or bytes.
///
/// `0` is allowed but be careful with infinite loops.
/// # Examples
/// ## For string (`&str`)
/// ```
/// # use whitehole::{combinator::{take, Combinator}, action::Action};
/// # fn t(_: Combinator<impl Action>) {}
/// # t(
/// take(10) // take 10 chars
/// # );
/// ```
/// ## For bytes (`&[u8]`)
/// ```
/// # use whitehole::{combinator::{take, Combinator}, action::Action};
/// # fn t(_: Combinator<impl Action>) {}
/// # t(
/// take(10) // take 10 bytes
/// # );
/// ```
#[inline]
pub const fn take(n: usize) -> Combinator<Take> {
  Combinator::new(Take::new(n))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::instant::Instant;

  #[test]
  fn test_take() {
    // normal
    assert_eq!(
      take(3)
        .exec(Input::new(Instant::new("123456"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(3)
    );
    assert_eq!(
      take(3)
        .exec(Input::new(
          Instant::new(b"123456" as &[u8]),
          &mut (),
          &mut ()
        ))
        .map(|output| output.digested),
      Some(3)
    );
    // reject
    assert!(take(7)
      .exec(Input::new(Instant::new("123456"), &mut (), &mut ()))
      .is_none());
    // 0 is always accepted
    assert_eq!(
      take(0)
        .exec(Input::new(Instant::new(""), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(0)
    );
    assert_eq!(
      take(0)
        .exec(Input::new(Instant::new("123456"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(0)
    );
    // take by chars not bytes for &str
    assert_eq!(
      take(1)
        .exec(Input::new(Instant::new("好"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(3)
    );
    assert_eq!(
      take(2)
        .exec(Input::new(Instant::new("好好"), &mut (), &mut ()))
        .map(|output| output.digested),
      Some(6)
    );
  }
}
