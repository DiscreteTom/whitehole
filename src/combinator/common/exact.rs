use crate::combinator::{Combinator, Output};

/// A util trait to make [`exact`] family generic over different types.
///
/// Built-in implementations are provided for [`String`], `&str`, and [`char`].
pub trait ExactPrefix {
  /// Check if the input starts with the prefix.
  fn is_prefix_of(&self, input: &str) -> bool;
  /// Get the byte length of the prefix.
  fn byte_len(&self) -> usize;
}

impl ExactPrefix for String {
  fn is_prefix_of(&self, input: &str) -> bool {
    input.starts_with(self)
  }
  fn byte_len(&self) -> usize {
    self.len()
  }
}

impl ExactPrefix for &str {
  fn is_prefix_of(&self, input: &str) -> bool {
    input.starts_with(self)
  }
  fn byte_len(&self) -> usize {
    self.len()
  }
}

impl ExactPrefix for char {
  fn is_prefix_of(&self, input: &str) -> bool {
    input.starts_with(*self)
  }
  fn byte_len(&self) -> usize {
    self.len_utf8()
  }
}

/// Match a prefix exactly, no lookahead.
/// Reject if not found.
///
/// Empty string is allowed, but be careful with infinite loops.
/// # Examples
/// ```
/// # use whitehole::combinator::{Combinator, exact};
/// let _: Combinator<_> = exact("true".to_string()); // with String
/// let _: Combinator<_> = exact("true"); // with &str
/// let _: Combinator<_> = exact(';'); // with char
/// ```
pub fn exact<'a, State, Heap>(prefix: impl ExactPrefix + 'a) -> Combinator<'a, (), State, Heap> {
  Combinator::boxed(move |input| {
    if !prefix.is_prefix_of(input.rest()) {
      return None;
    }

    Output {
      kind: (),
      digested: prefix.byte_len(), // might be 0
    }
    .into()
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
