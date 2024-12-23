use crate::{
  action::{Input, Output},
  combinator::wrap,
  C,
};

/// A util trait to make [`till`] generic over different types.
///
/// Built-in implementations are provided for [`String`], `&str`, [`char`] and `()`.
///
/// See [`till`] for more details.
pub trait Till<State, Heap> {
  /// Check if the rest of input text contains this instance.
  /// Return the output after digesting the instance if found.
  fn exec<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, ()>>;
}

impl<State, Heap> Till<State, Heap> for &str {
  #[inline]
  fn exec<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, ()>> {
    input
      .rest()
      .find(self)
      .map(|i| unsafe { input.digest_unchecked(i + self.len()) })
  }
}

impl<State, Heap> Till<State, Heap> for String {
  #[inline]
  fn exec<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, ()>> {
    self.as_str().exec(input)
  }
}

impl<State, Heap> Till<State, Heap> for char {
  #[inline]
  fn exec<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, ()>> {
    input
      .rest()
      .find(*self)
      .map(|i| unsafe { input.digest_unchecked(i + self.len_utf8()) })
  }
}

impl<State, Heap> Till<State, Heap> for () {
  #[inline]
  fn exec<'text>(&self, _: &mut Input<'text, &mut State, &mut Heap>) -> Option<Output<'text, ()>> {
    Some(Output {
      value: (),
      rest: "",
    })
  }
}

/// Return a combinator to match the provided pattern, eat all the bytes
/// to the end of the first occurrence of the pattern (inclusive).
///
/// Empty string is allowed, but be careful with infinite loops.
/// # Examples
/// ```
/// # use whitehole::{combinator::till, Combinator};
/// # fn t(_: C!()) {}
/// # t(
/// till("end".to_string()) // with String
/// # );
/// # t(
/// till("end") // with &str
/// # );
/// # t(
/// till(';') // with char
/// # );
/// # t(
/// till(()) // with (), eat all rest
/// # );
/// ```
#[inline]
pub const fn till<State, Heap>(pattern: impl Till<State, Heap>) -> C!((), State, Heap) {
  wrap(move |input| pattern.exec(input))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::action::{Action, Input, Output};

  #[test]
  fn until_exec() {
    assert_eq!(
      till("end".to_string()).exec(&mut Input::new("123end456", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "456"
      })
    );
    assert_eq!(
      till("end").exec(&mut Input::new("123end456", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "456"
      })
    );
    assert_eq!(
      till(';').exec(&mut Input::new("123;456", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: "456"
      })
    );
    assert_eq!(
      till(()).exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        rest: ""
      })
    );
  }
}
