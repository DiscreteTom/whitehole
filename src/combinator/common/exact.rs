use crate::{
  combinator::{Combinator, Input, Output},
  impl_combinator_ops,
};

/// A util trait to make [`exact`] generic over different types.
///
/// Built-in implementations are provided for [`String`], `&str`, and [`char`].
///
/// See [`exact`] for more details.
/// # Safety
/// You should ensure that [`Output::rest`] can be built
/// as a valid UTF-8 string.
/// This will be checked using [`debug_assert!`].
pub unsafe trait Exact {
  /// Check if the input starts with this instance.
  /// Return the length of digested bytes if found.
  fn parse(&self, input: &str) -> Option<usize>;
}

unsafe impl Exact for String {
  #[inline]
  fn parse(&self, input: &str) -> Option<usize> {
    input.starts_with(self).then_some(self.len())
  }
}

unsafe impl Exact for &str {
  #[inline]
  fn parse(&self, input: &str) -> Option<usize> {
    input.starts_with(self).then_some(self.len())
  }
}

unsafe impl Exact for char {
  #[inline]
  fn parse(&self, input: &str) -> Option<usize> {
    input.starts_with(*self).then(|| self.len_utf8())
  }
}

/// See [`exact`].
#[derive(Debug, Clone, Copy)]
pub struct ExactCombinator<P> {
  pattern: P,
}

/// Match a pattern exactly, no lookahead.
/// Reject if not found.
///
/// Empty string is allowed, but be careful with infinite loops.
/// # Examples
/// ```
/// # use whitehole::combinator::exact;
/// exact("true".to_string()); // with String
/// exact("true"); // with &str
/// exact(';'); // with char
///
/// // to lookahead one char to ensure there is a word boundary,
/// // use the `boundary` decorator
/// exact("true").boundary();
/// ```
#[inline]
pub fn exact<P: Exact>(pattern: P) -> ExactCombinator<P> {
  ExactCombinator { pattern }
}

impl<State, Heap, P: Exact> Combinator<State, Heap> for ExactCombinator<P> {
  type Kind = ();

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, Self::Kind>> {
    self
      .pattern
      .parse(input.rest())
      .map(|digested| unsafe { input.digest_unchecked(digested) })
  }
}

impl_combinator_ops!(ExactCombinator<P>, P);

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::Input;

  #[test]
  fn combinator_exact() {
    // normal str
    assert_eq!(
      exact("123")
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
    // normal String
    assert_eq!(
      exact("123".to_string())
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
    // normal char
    assert_eq!(
      exact(';')
        .parse(&mut Input::new(";", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("")
    );
    // reject
    assert!(exact("123")
      .parse(&mut Input::new("12", 0, &mut (), &mut ()).unwrap())
      .is_none());
    // empty
    assert_eq!(
      exact("")
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.rest),
      Some("123")
    );
  }
}
