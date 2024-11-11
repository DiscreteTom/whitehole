use crate::combinator::{Combinator, Output};

/// A util trait to make [`till`] generic over different types.
///
/// Built-in implementations are provided for [`String`], `&str`, [`char`] and `()`.
///
/// See [`till`] for more details.
pub trait Till {
  /// Check if the input contains this instance.
  /// Return the rest of input if found.
  fn parse<'text>(&self, input: &'text str) -> Option<&'text str>;
}

impl Till for String {
  fn parse<'text>(&self, input: &'text str) -> Option<&'text str> {
    input
      .find(self)
      .map(|i| unsafe { input.get_unchecked(i + self.len()..) })
  }
}

impl Till for &str {
  fn parse<'text>(&self, input: &'text str) -> Option<&'text str> {
    input
      .find(self)
      .map(|i| unsafe { input.get_unchecked(i + self.len()..) })
  }
}

impl Till for char {
  fn parse<'text>(&self, input: &'text str) -> Option<&'text str> {
    input
      .find(*self)
      .map(|i| unsafe { input.get_unchecked(i + self.len_utf8()..) })
  }
}

impl Till for () {
  fn parse<'text>(&self, _: &'text str) -> Option<&'text str> {
    Some("")
  }
}

/// Match a pattern, eat all the bytes
/// to the end of the first occurrence of the pattern.
///
/// Empty string is allowed, but be careful with infinite loops.
/// # Examples
/// ```
/// # use whitehole::combinator::{Combinator, till};
/// let _: Combinator<_> = till("end".to_string()); // with String
/// let _: Combinator<_> = till("end"); // with &str
/// let _: Combinator<_> = till(';'); // with char
/// let _: Combinator<_> = till(()); // with (), eat all rest
/// ```
pub fn till<'a, State, Heap>(pattern: impl Till + 'a) -> Combinator<'a, (), State, Heap> {
  Combinator::boxed(move |input| {
    pattern
      .parse(input.rest())
      .map(|rest| Output { kind: (), rest })
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::Input;

  #[test]
  fn till_parse() {
    assert_eq!(
      till("end".to_string()).parse(&mut Input::new("123end456", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (),
        rest: "456"
      })
    );
    assert_eq!(
      till("end").parse(&mut Input::new("123end456", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (),
        rest: "456"
      })
    );
    assert_eq!(
      till(';').parse(&mut Input::new("123;456", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (),
        rest: "456"
      })
    );
    assert_eq!(
      till(()).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output { kind: (), rest: "" })
    );
  }
}
