/// [`Combinator`](crate::combinator::Combinator)'s output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output<Kind> {
  /// The [`Node::kind`](crate::node::Node::kind).
  pub kind: Kind,
  /// How many bytes are digested by this combinator.
  /// # Caveats
  /// `0` is allowed, but be careful with infinite loops.
  ///
  /// This value should be smaller than or equal to the length of
  /// [`Input::rest`](crate::combinator::input::Input::rest).
  pub digested: usize,
}

impl<Kind> Output<Kind> {
  /// Convert [`Self::kind`] to a new kind.
  pub fn map<NewKind>(self, f: impl FnOnce(Kind) -> NewKind) -> Output<NewKind> {
    Output {
      kind: f(self.kind),
      digested: self.digested,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn output_map() {
    assert_eq!(
      Output {
        kind: 1,
        digested: 2,
      }
      .map(|kind| kind + 1),
      Output {
        kind: 2,
        digested: 2,
      }
    );
  }
}
