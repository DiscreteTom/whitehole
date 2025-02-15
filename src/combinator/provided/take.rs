use crate::{
  action::{Action, Context, Output},
  combinator::Combinator,
  instant::Instant,
};

/// See [`take`].
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
  fn exec(&self, instant: Instant<&str>, _: Context<&mut State, &mut Heap>) -> Option<Output<()>> {
    let mut digested: usize = 0;
    let mut count: usize = 0;
    let mut chars = instant.rest().chars();
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
    unsafe { instant.accept_unchecked(digested) }.into()
  }
}

unsafe impl<State, Heap> Action<[u8], State, Heap> for Take {
  type Value = ();

  #[inline]
  fn exec(&self, instant: Instant<&[u8]>, _: Context<&mut State, &mut Heap>) -> Option<Output<()>> {
    instant.accept(self.n)
  }
}

/// Returns a combinator to take the next `n` undigested [`char`]s or bytes.
///
/// `0` is allowed but be careful with infinite loops.
/// # Examples
/// For string (`&str`):
/// ```
/// # use whitehole::{combinator::{take, Combinator}, action::Action};
/// # fn t(_: Combinator<impl Action>) {}
/// # t(
/// take(10) // take 10 chars
/// # );
/// ```
/// For bytes (`&[u8]`):
/// ```
/// # use whitehole::{combinator::{take, Combinator}, action::Action};
/// # fn t(_: Combinator<impl Action<[u8]>>) {}
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
        .exec(Instant::new("123456"), Context::default())
        .map(|output| output.digested),
      Some(3)
    );
    assert_eq!(
      take(3)
        .exec(Instant::new(b"123456" as &[u8]), Context::default())
        .map(|output| output.digested),
      Some(3)
    );
    // reject
    assert!(take(7)
      .exec(Instant::new("123456"), Context::default())
      .is_none());
    // 0 is always accepted
    assert_eq!(
      take(0)
        .exec(Instant::new(""), Context::default())
        .map(|output| output.digested),
      Some(0)
    );
    assert_eq!(
      take(0)
        .exec(Instant::new("123456"), Context::default())
        .map(|output| output.digested),
      Some(0)
    );
    // take by chars not bytes for &str
    assert_eq!(
      take(1)
        .exec(Instant::new("好"), Context::default())
        .map(|output| output.digested),
      Some(3)
    );
    assert_eq!(
      take(2)
        .exec(Instant::new("好好"), Context::default())
        .map(|output| output.digested),
      Some(6)
    );
  }
}
