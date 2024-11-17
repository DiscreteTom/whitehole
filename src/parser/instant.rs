/// The instantaneous state of a parser (a.k.a the "configuration" in the automata theory).
///
/// This is cheap to clone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instant<'text> {
  /// See [`Self::text`].
  text: &'text str,
  /// See [`Self::rest`].
  rest: &'text str,
  /// See [`Self::digested`].
  digested: usize,
}

impl<'text> Instant<'text> {
  /// Create a new instance with the given text.
  /// [`Self::digested`] will be set to `0`.
  #[inline]
  pub const fn new(text: &'text str) -> Self {
    Instant {
      text,
      rest: text,
      digested: 0,
    }
  }

  /// The whole input text.
  #[inline]
  pub const fn text(&self) -> &'text str {
    self.text
  }
  /// How many bytes are already digested.
  #[inline]
  pub const fn digested(&self) -> usize {
    self.digested
  }
  /// The undigested text.
  #[inline]
  pub const fn rest(&self) -> &'text str {
    self.rest
  }

  /// Update with the provided rest.
  /// [`Self::digested`] will be auto calculated.
  #[inline]
  pub fn update(&mut self, rest: &'text str) {
    self.rest = rest;
    self.digested = self.text.len() - rest.len();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn instant_new() {
    let state = Instant::new("123");
    assert_eq!(state.digested(), 0);
    assert_eq!(state.rest(), "123");
    assert_eq!(state.text(), "123");
  }

  #[test]
  fn instant_update() {
    let mut state = Instant::new("123");
    state.update("3");
    assert_eq!(state.digested(), 2);
    assert_eq!(state.rest(), "3");
    assert_eq!(state.text(), "123");
  }
}
