use crate::{
  combinator::{Parse, Input, Output},
  impl_combinator_ops,
};

/// A util trait to make [`till`] generic over different types.
///
/// Built-in implementations are provided for [`String`], `&str`, [`char`] and `()`.
///
/// See [`till`] for more details.
/// # Safety
/// You should ensure that [`Output::rest`](crate::combinator::Output::rest) can be built
/// as a valid UTF-8 string.
/// This will be checked using [`debug_assert!`].
pub unsafe trait Till {
  /// Check if the input contains this instance.
  /// Return the length of digested bytes if found.
  fn parse(&self, input: &str) -> Option<usize>;
}

unsafe impl Till for String {
  #[inline]
  fn parse(&self, input: &str) -> Option<usize> {
    input.find(self).map(|i| i + self.len())
  }
}

unsafe impl Till for &str {
  #[inline]
  fn parse(&self, input: &str) -> Option<usize> {
    input.find(self).map(|i| i + self.len())
  }
}

unsafe impl Till for char {
  #[inline]
  fn parse(&self, input: &str) -> Option<usize> {
    input.find(*self).map(|i| i + self.len_utf8())
  }
}

unsafe impl Till for () {
  #[inline]
  fn parse(&self, input: &str) -> Option<usize> {
    Some(input.len())
  }
}

/// See [`till`].
#[derive(Debug, Clone, Copy)]
pub struct TillCombinator<T> {
  pattern: T,
}

/// Match a pattern, eat all the bytes
/// to the end of the first occurrence of the pattern.
///
/// Empty string is allowed, but be careful with infinite loops.
/// # Examples
/// ```
/// # use whitehole::combinator::till;
/// till("end".to_string()); // with String
/// till("end"); // with &str
/// till(';'); // with char
/// till(()); // with (), eat all rest
/// ```
#[inline]
pub fn till<T: Till>(pattern: T) -> TillCombinator<T> {
  TillCombinator { pattern }
}

impl<State, Heap, T: Till> Parse<State, Heap> for TillCombinator<T> {
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

impl_combinator_ops!(TillCombinator<T>, T);

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::{Input, Output};

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
