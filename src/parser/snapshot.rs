/// The snapshot of [`Parser`](crate::parser::Parser).
///
/// This can be created by [`Parser::snapshot`](crate::parser::Parser::snapshot)
/// and used by [`Parser::restore`](crate::parser::Parser::restore).
///
/// You can't construct this manually because
/// you shouldn't modify [`Self::digested`] and [`Self::text`] directly.
///
/// Since `State` should be cheap to clone,
/// this is also cheap to create or clone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot<'text, State> {
  /// See [`Parser::state`](crate::parser::Parser::state).
  /// You can modify this if needed.
  pub state: State,
  /// See [`Parser::text`](crate::parser::Parser::text).
  pub(super) text: &'text str,
  /// See [`Parser::digested`](crate::parser::Parser::digested).
  pub(super) digested: usize,
}

impl<'text, State> Snapshot<'text, State> {
  /// See [`Parser::text`](crate::parser::Parser::text).
  /// You can't modify this manually.
  pub const fn text(&self) -> &'text str {
    self.text
  }

  /// See [`Parser::digested`](crate::parser::Parser::digested).
  /// You can't modify this manually.
  pub const fn digested(&self) -> usize {
    self.digested
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_snapshot() {
    assert_eq!(
      Snapshot {
        state: (),
        text: "123",
        digested: 1
      }
      .text(),
      "123"
    );
    assert_eq!(
      Snapshot {
        state: (),
        text: "123",
        digested: 1
      }
      .digested(),
      1
    );
  }
}
