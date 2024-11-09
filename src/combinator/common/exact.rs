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

/// Match a word,
/// lookahead one char to ensure there is a word boundary
/// (non-alphanumeric and not `_`) or end of input after the word.
/// Reject if not found.
///
/// Empty string is allowed, but be careful with infinite loops.
/// You can use `word("")` as a word boundary checker.
/// # Examples
/// ```
/// # use whitehole::combinator::{Combinator, word};
/// let _: Combinator<_> = word("int".to_string()); // with String
/// let _: Combinator<_> = word("int"); // with &str
/// let _: Combinator<_> = word('i'); // with char
/// ```
pub fn word<'a, State, Heap>(word: impl ExactPrefix + 'a) -> Combinator<'a, (), State, Heap> {
  Combinator::boxed(move |input| {
    if !word.is_prefix_of(input.rest()) {
      return None;
    }

    // reject if next char exists and is alphanumeric or `_`
    if input
      .digest(word.byte_len())
      .map(|input| {
        let next = input.next();
        next.is_alphanumeric() || next == '_'
      })
      .unwrap_or(false)
    {
      return None;
    }

    Output {
      kind: (),
      digested: word.byte_len(), // might be 0
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

  #[test]
  fn combinator_word() {
    // normal str
    assert_eq!(
      word("123")
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // normal String
    assert_eq!(
      word("123".to_string())
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // normal char
    assert_eq!(
      word('1')
        .parse(&mut Input::new("1", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(1)
    );
    // reject
    assert!(word("123")
      .parse(&mut Input::new("12", 0, &mut (), &mut ()).unwrap())
      .is_none());
    // empty
    assert_eq!(
      word("")
        .parse(&mut Input::new("-123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(0)
    );
    assert_eq!(
      word("")
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      None
    );
    // reject if next char is alphanumeric
    assert!(word("123")
      .parse(&mut Input::new("1234", 0, &mut (), &mut ()).unwrap())
      .is_none());
    // reject if next char is `_`
    assert!(word("123")
      .parse(&mut Input::new("123_", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }
}
