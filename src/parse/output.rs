/// The output of [`Parse::parse`](crate::parse::Parse::parse).
///
/// Usually built by [`Input::digest`](crate::parse::Input::digest).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output<'text, Kind> {
  /// The [`Node::kind`](crate::node::Node::kind).
  pub kind: Kind,
  /// The rest of the input text.
  pub rest: &'text str,
}

impl<'text, Kind> Output<'text, Kind> {
  /// Convert [`Self::kind`] to a new kind.
  #[inline]
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
