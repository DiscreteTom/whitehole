use super::instant::Instant;

/// The snapshot of the stateful [`Lexer`](crate::lexer::Lexer).
///
/// This can be created by [`Lexer::snapshot`](crate::lexer::Lexer::snapshot)
/// and use by [`Lexer::restore`](crate::lexer::Lexer::restore).
///
/// You can't construct this manually because
/// you shouldn't modify [`Self::instant`] directly.
///
/// Since `State` should be cheap to clone,
/// this is also cheap to clone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot<'text, State> {
  /// See [`Lexer::state`](crate::lexer::Lexer::state).
  pub state: State,
  /// See [`Self::instant`].
  pub(super) instant: Instant<'text>,
}

impl<'text, State> Snapshot<'text, State> {
  /// Get the [`Lexer::instant`](crate::lexer::Lexer::instant)
  /// in the [`Snapshot`].
  /// You can't modify this manually.
  #[inline]
  pub const fn instant(&self) -> &Instant<'text> {
    &self.instant
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
        instant: Instant::new("")
      }
      .instant(),
      &Instant::new("")
    );
  }
}
