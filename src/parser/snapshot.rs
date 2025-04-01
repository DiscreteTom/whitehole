use super::Instant;

/// The snapshot of a [`Parser`](crate::parser::Parser).
///
/// This can be created by [`Parser::snapshot`](crate::parser::Parser::snapshot)
/// and used by [`Parser::restore`](crate::parser::Parser::restore).
///
/// Since `State` should be cheap to clone,
/// this is also cheap to create or clone.
#[derive(Debug, Clone)]
pub struct Snapshot<TextRef, State> {
  /// See [`Parser::state`](crate::parser::Parser::state).
  /// You can modify this if needed.
  pub state: State,

  /// See [`Parser::instant`](crate::parser::Parser::instant).
  /// You can modify this if needed.
  pub instant: Instant<TextRef>,
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
