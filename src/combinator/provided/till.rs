use crate::{
  action::{Action, Input, Output},
  combinator::{create_value_combinator, Combinator},
};

create_value_combinator!(Till, "See [`till`].");

unsafe impl<State, Heap> Action for Till<&str, State, Heap> {
  type Value = ();
  type State = State;
  type Heap = Heap;

  #[inline]
  fn exec(&self, input: Input<&mut State, &mut Heap>) -> Option<Output<()>> {
    input
      .instant()
      .rest()
      .find(self.inner)
      .map(|i| unsafe { input.digest_unchecked(i.unchecked_add(self.inner.len())) })
  }
}

unsafe impl<State, Heap> Action for Till<String, State, Heap> {
  type Value = ();
  type State = State;
  type Heap = Heap;

  #[inline]
  fn exec(&self, input: Input<&mut State, &mut Heap>) -> Option<Output<()>> {
    input
      .instant()
      .rest()
      .find(&self.inner)
      .map(|i| unsafe { input.digest_unchecked(i.unchecked_add(self.inner.len())) })
  }
}

unsafe impl<State, Heap> Action for Till<char, State, Heap> {
  type Value = ();
  type State = State;
  type Heap = Heap;

  #[inline]
  fn exec(&self, input: Input<&mut State, &mut Heap>) -> Option<Output<()>> {
    input
      .instant()
      .rest()
      .find(self.inner)
      .map(|i| unsafe { input.digest_unchecked(i.unchecked_add(self.inner.len_utf8())) })
  }
}

unsafe impl<State, Heap> Action for Till<(), State, Heap> {
  type Value = ();
  type State = State;
  type Heap = Heap;

  #[inline]
  fn exec(&self, input: Input<&mut State, &mut Heap>) -> Option<Output<()>> {
    unsafe { input.digest_unchecked(input.instant().rest().len()) }.into()
  }
}

/// Return a combinator to match the provided pattern, eat all the bytes
/// to the end of the first occurrence of the pattern (inclusive).
///
/// Empty string is allowed, but be careful with infinite loops.
/// # Examples
/// ```
/// # use whitehole::{combinator::till, C};
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
pub const fn till<State, Heap, T>(pattern: T) -> Combinator<Till<T, State, Heap>> {
  Combinator::new(Till::new(pattern))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    action::{Action, Input, Output},
    instant::Instant,
  };

  #[test]
  fn until_exec() {
    assert_eq!(
      till("end".to_string())
        .exec(Input::new(Instant::new("123end456"), &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        digested: 6
      })
    );
    assert_eq!(
      till("end").exec(Input::new(Instant::new("123end456"), &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        digested: 6
      })
    );
    assert_eq!(
      till(';').exec(Input::new(Instant::new("123;456"), &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        digested: 4
      })
    );
    assert_eq!(
      till(()).exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap()),
      Some(Output {
        value: (),
        digested: 3
      })
    );
  }
}
