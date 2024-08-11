use super::{instant::Instant, Lexer};

/// The snapshot of the stateful [`Lexer`].
///
/// This can be created by [`Lexer::snapshot`]
/// and use by [`Lexer::restore`] and [`Lexer::clone_with_snapshot`].
///
/// You can't construct this manually because
/// you shouldn't modify [`Self::instant`] directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot<'text, State> {
  pub state: State,
  pub(super) instant: Instant<'text>,
}

impl<'text, State> Snapshot<'text, State> {
  /// Get the instant of the snapshot.
  #[inline]
  pub const fn instant(&self) -> &Instant<'text> {
    &self.instant
  }
}

/// Partial [`Snapshot`].
///
/// This can be turned into a full [`Snapshot`] by [`Self::into_full`].
///
/// You can construct this manually because
/// you shouldn't modify [`Self::instant`] directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialSnapshot<'text, State> {
  pub state: Option<State>,
  pub(super) instant: Option<Instant<'text>>,
}

impl<'text, State> From<Snapshot<'text, State>> for PartialSnapshot<'text, State> {
  #[inline]
  fn from(snapshot: Snapshot<'text, State>) -> Self {
    PartialSnapshot {
      state: Some(snapshot.state),
      instant: Some(snapshot.instant),
    }
  }
}

impl<'text, State> PartialSnapshot<'text, State> {
  /// Get the instant of the partial snapshot.
  #[inline]
  pub const fn instant(&self) -> &Option<Instant<'text>> {
    &self.instant
  }

  /// Consume self, build a full [`Snapshot`].
  /// This will clone [`Lexer::state`] and/or [`Lexer::instant`] if they are [`None`].
  #[inline]
  pub fn into_full<Kind, ErrorType>(
    self,
    lexer: &Lexer<'text, Kind, State, ErrorType>,
  ) -> Snapshot<'text, State>
  where
    State: Clone,
  {
    Snapshot {
      state: self.state.unwrap_or_else(|| lexer.state.clone()),
      instant: self.instant.unwrap_or_else(|| lexer.instant().clone()),
    }
  }
}
