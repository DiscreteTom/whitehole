use super::{Combinator, Output};
use std::ops::{Add, BitOr, Mul};

impl<'a, Kind: 'a, State: 'a, Heap: 'a> BitOr for Combinator<'a, Kind, State, Heap> {
  type Output = Self;

  /// Try to parse with the left-hand side, if it fails, try the right-hand side.
  fn bitor(self, rhs: Self) -> Self::Output {
    Combinator::boxed(move |input| self.parse(input).or_else(|| rhs.parse(input)))
  }
}

impl<'a, Kind: 'a, State: 'a, Heap: 'a, NewKind: 'a> Add<Combinator<'a, NewKind, State, Heap>>
  for Combinator<'a, Kind, State, Heap>
{
  type Output = Combinator<'a, NewKind, State, Heap>;

  /// Parse with the left-hand side, then parse with the right-hand side.
  /// Return the output with the kind of the right hand side and the sum of the digested.
  fn add(self, rhs: Combinator<'a, NewKind, State, Heap>) -> Self::Output {
    Combinator::boxed(move |input| {
      self.parse(input).and_then(|output| {
        input
          .digest(output.digested)
          .and_then(|mut input| rhs.parse(&mut input))
          .map(|rhs_output| Output {
            kind: rhs_output.kind,
            digested: output.digested + rhs_output.digested,
          })
      })
    })
  }
}

impl<'a, Kind: 'a, State: 'a, Heap: 'a> Mul<usize> for Combinator<'a, Kind, State, Heap> {
  type Output = Combinator<'a, Kind, State, Heap>;

  /// Repeat the combinator `rhs` times.
  /// Return the output with the kind of the last output and the sum of the digested.
  fn mul(self, rhs: usize) -> Self::Output {
    Combinator::boxed(move |input| {
      if rhs == 0 {
        return None;
      }

      self.parse(input).and_then(|mut output| {
        for _ in 1..rhs {
          let next_output = self.parse(&mut input.digest(output.digested)?)?;
          output.digested += next_output.digested;
          output.kind = next_output.kind;
        }
        output.into()
      })
    })
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

  #[test]
  fn combinator_add() {
    let rejecter = || Combinator::boxed(|_| Option::<Output<()>>::None);
    let accepter_unit = || {
      Combinator::boxed(|_| {
        Some(Output {
          kind: (),
          digested: 1,
        })
      })
    };
    let accepter_int = || {
      Combinator::boxed(|_| {
        Some(Output {
          kind: 123,
          digested: 1,
        })
      })
    };

    // reject then accept, should return None
    assert!((rejecter() + accepter_unit())
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // accept then reject, should return None
    assert!((accepter_unit() + rejecter())
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // accept then accept, should return the sum of the digested
    // with the kind of the right-hand side
    assert_eq!(
      (accepter_unit() + accepter_int())
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: 123,
        digested: 2,
      })
    );
    assert_eq!(
      (accepter_int() + accepter_unit())
        .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: (),
        digested: 2,
      })
    );
  }

  #[test]
  fn combinator_mul() {
    let rejecter = || Combinator::boxed(|_| Option::<Output<()>>::None);
    let accepter = || {
      Combinator::boxed(|input| {
        Some(Output {
          kind: input.next(),
          digested: 1,
        })
      })
    };

    // repeat a rejecter will reject
    assert!((rejecter() * 3)
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // repeat an accepter 0 times will reject
    let n = 0;
    assert!((accepter() * n)
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());

    // normal, apply the last output's kind and sum the digested
    assert_eq!(
      (accepter() * 3).parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()),
      Some(Output {
        kind: '3',
        digested: 3,
      })
    );

    // overflow, reject
    assert!((accepter() * 4)
      .parse(&mut Input::new("123", 0, &mut (), &mut ()).unwrap())
      .is_none());
  }
}
