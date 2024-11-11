use crate::combinator::{Combinator, Output};

/// A util trait to make [`exact`] generic over different types.
///
/// Built-in implementations are provided for [`String`], `&str`, and [`char`].
///
/// See [`exact`] for more details.
pub trait Exact {
  /// Check if the input starts with this instance.
  /// Return the byte length of the prefix if found.
  /// `0` is allowed, but be careful with infinite loops.
  fn parse(&self, input: &str) -> Option<usize>;
}

impl Exact for String {
  fn parse(&self, input: &str) -> Option<usize> {
    input.starts_with(self).then_some(self.len())
  }
}

impl Exact for &str {
  fn parse(&self, input: &str) -> Option<usize> {
    input.starts_with(self).then_some(self.len())
  }
}

impl Exact for char {
  fn parse(&self, input: &str) -> Option<usize> {
    input.starts_with(*self).then(|| self.len_utf8())
  }
}

/// Match a pattern exactly, no lookahead.
/// Reject if not found.
///
/// Empty string is allowed, but be careful with infinite loops.
/// # Examples
/// ```
/// # use whitehole::combinator::{Combinator, exact};
/// let _: Combinator<_> = exact("true".to_string()); // with String
/// let _: Combinator<_> = exact("true"); // with &str
/// let _: Combinator<_> = exact(';'); // with char
///
/// // to lookahead one char to ensure there is a word boundary,
/// // use the `boundary` decorator
/// let _: Combinator<_> = exact("true").boundary();
/// ```
pub fn exact<'a, State, Heap>(pattern: impl Exact + 'a) -> Combinator<'a, (), State, Heap> {
  Combinator::boxed(move |input| {
    pattern
      .parse(input.rest())
      .map(|digested| Output { kind: (), digested })
  })
}

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
        .map(|output| output.digested),
      Some(3)
    );
    // normal String
    assert_eq!(
      exact("123".to_string())
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // normal char
    assert_eq!(
      exact(';')
        .parse(&mut Input::new(";", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(1)
    );
    // reject
    assert!(exact("123")
      .parse(&mut Input::new("12", 0, &mut (), &mut ()).unwrap())
      .is_none());
    // empty
    assert_eq!(
      exact("")
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(0)
    );
  }
}
