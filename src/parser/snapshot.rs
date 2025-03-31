use super::Instant;

/// The snapshot of a [`Parser`](crate::parser::Parser).
///
/// This can be created by [`Parser::snapshot`](crate::parser::Parser::snapshot)
/// and used by [`Parser::restore`](crate::parser::Parser::restore).
///
/// You can't construct this manually because
/// you shouldn't modify [`Self::instant`] directly.
///
/// Since `State` should be cheap to clone,
/// this is also cheap to create or clone.
#[derive(Debug, Clone)]
pub struct Snapshot<TextRef, State> {
  /// See [`Parser::state`](crate::parser::Parser::state).
  /// You can modify this if needed.
  pub state: State,
  /// See [`Self::instant`].
  pub(super) instant: Instant<TextRef>,
}

impl<TextRef, State> Snapshot<TextRef, State> {
  /// See [`Parser::instant`](crate::parser::Parser::instant).
  /// You can't modify this manually.
  #[inline]
  pub const fn instant(&self) -> &Instant<TextRef> {
    &self.instant
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn _test_snapshot() {
    let s = Snapshot {
      state: (),
      instant: Instant::new(""),
    };

    // debug
    let _ = format!("{:?}", s);
    // ensure clone-able
    let _ = s.clone();
  }
}
