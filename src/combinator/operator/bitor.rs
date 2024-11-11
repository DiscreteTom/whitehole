//! Overload [`BitOr`] operator for [`Combinator`].

use crate::combinator::{eat, exact, Combinator, Exact};
use std::ops::BitOr;

impl<'a, Kind: 'a, State: 'a, Heap: 'a> BitOr for Combinator<'a, Kind, State, Heap> {
  type Output = Self;

  /// Try to parse with the left-hand side, if it fails, try the right-hand side.
  fn bitor(self, rhs: Self) -> Self::Output {
    Combinator::boxed(move |input| self.parse(input).or_else(|| rhs.parse(input)))
  }
}

impl<'a, State: 'a, Heap: 'a, T: Exact + 'a> BitOr<T> for Combinator<'a, (), State, Heap> {
  type Output = Combinator<'a, (), State, Heap>;

  /// Shortcut for `self | exact(rhs)`. See [`exact`].
  fn bitor(self, rhs: T) -> Self::Output {
    self | exact(rhs)
  }
}

impl<'a, State: 'a, Heap: 'a> BitOr<usize> for Combinator<'a, (), State, Heap> {
  type Output = Combinator<'a, (), State, Heap>;

  /// Shortcut for `self | eat(rhs)`. See [`eat`].
  fn bitor(self, rhs: usize) -> Self::Output {
    self | eat(rhs)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::{Input, Output};

  #[test]
  fn combinator_bit_or() {
    let mut state = 0;

    let rejecter = || {
      Combinator::boxed(|input| {
        *input.state += 1;
        None
      })
    };
    let accepter = || {
      Combinator::boxed(|input| {
        *input.state += 1;
        Some(Output {
          kind: (),
          digested: 1,
        })
      })
    };

    // reject then accept, both should increment the state
    assert_eq!(
      (rejecter() | accepter()).parse(&mut Input::new("123", 0, &mut state, &mut ()).unwrap()),
      Some(Output {
        kind: (),
        digested: 1,
      })
    );
    assert_eq!(state, 2);

    state = 0;

    // accept then reject, only the first should increment the state
    assert_eq!(
      (accepter() | rejecter()).parse(&mut Input::new("123", 0, &mut state, &mut ()).unwrap()),
      Some(Output {
        kind: (),
        digested: 1,
      })
    );
    assert_eq!(state, 1);
  }

  fn _combinator_bit_or_exact_prefix() {
    let _: Combinator<_> = Combinator::boxed(|_| None) | "123"; // with &str
    let _: Combinator<_> = Combinator::boxed(|_| None) | "123".to_string(); // with String
    let _: Combinator<_> = Combinator::boxed(|_| None) | '1'; // with char
  }

  fn _combinator_bit_or_usize() {
    let _: Combinator<_> = Combinator::boxed(|_| None) | 1;
  }
}
