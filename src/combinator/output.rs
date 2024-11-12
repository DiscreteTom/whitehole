/// [`Combinator`](crate::combinator::Combinator)'s output.
///
/// Usually built by [`Input::digest`](crate::combinator::Input::digest).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output<'text, Kind> {
  /// The [`Node::kind`](crate::node::Node::kind).
  pub kind: Kind,
  /// The rest of the input text.
  pub rest: &'text str,
}

impl<'text, Kind> Output<'text, Kind> {
  /// Convert [`Self::kind`] to a new kind.
  pub fn map<NewKind>(self, f: impl FnOnce(Kind) -> NewKind) -> Output<'text, NewKind> {
    Output {
      kind: f(self.kind),
      rest: self.rest,
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
        rest: "123",
      }
      .map(|kind| kind + 1),
      Output {
        kind: 2,
        rest: "123",
      }
    );
  }
}
