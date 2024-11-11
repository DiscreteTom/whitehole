use crate::combinator::{Combinator, Output};

/// A util trait to make [`till`] generic over different types.
///
/// Built-in implementations are provided for [`String`], `&str`, [`char`] and `()`.
///
/// See [`till`] for more details.
pub trait Till {
  /// Check if the input contains this instance.
  /// Return the total byte length from the start of the `input`
  /// to the end of the first occurrence of this instance if found.
  /// `0` is allowed, but be careful with infinite loops.
  fn parse(&self, input: &str) -> Option<usize>;
}

impl Till for String {
  fn parse(&self, input: &str) -> Option<usize> {
    input.find(self).map(|i| i + self.len())
  }
}

impl Till for &str {
  fn parse(&self, input: &str) -> Option<usize> {
    input.find(self).map(|i| i + self.len())
  }
}

impl Till for char {
  fn parse(&self, input: &str) -> Option<usize> {
    input.find(*self).map(|i| i + self.len_utf8())
  }
}

impl Till for () {
  fn parse(&self, input: &str) -> Option<usize> {
    Some(input.len())
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
      .map(|digested| Output { kind: (), digested })
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
        digested: 6
      })
    );
    assert_eq!(
      till("end").parse(&mut Input::new("123end456", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (),
        digested: 6
      })
    );
    assert_eq!(
      till(';').parse(&mut Input::new("123;456", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (),
        digested: 4
      })
    );
    assert_eq!(
      till(()).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (),
        digested: 3
      })
    );
  }
}
