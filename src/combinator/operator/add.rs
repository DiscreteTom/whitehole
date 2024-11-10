use crate::combinator::{Combinator, Output};
use std::ops::Add;

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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::combinator::Input;

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
}
