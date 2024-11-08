use super::Combinator;
use std::ops;

impl<'a, Kind: 'a, State: 'a, Heap: 'a> ops::BitOr for Combinator<'a, Kind, State, Heap> {
  type Output = Self;

  /// Try to parse with the left-hand side, if it fails, try the right-hand side.
  fn bitor(self, rhs: Self) -> Self::Output {
    Combinator {
      exec: Box::new(move |input| self.parse(input).or_else(|| rhs.parse(input))),
    }
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
}
