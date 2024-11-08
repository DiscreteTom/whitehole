use crate::combinator::generic;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputContext<'text> {
  /// See [`Input::text`].
  text: &'text str,
  /// See [`Input::rest`].
  rest: &'text str,
}

pub type Input<'text, StateRef, HeapRef> = generic::Input<InputContext<'text>, StateRef, HeapRef>;

impl<'text, StateRef, HeapRef> Input<'text, StateRef, HeapRef> {
  /// Return [`Some`] if [`Self::rest`] can be constructed and not empty.
  pub fn new(text: &'text str, start: usize, state: StateRef, heap: HeapRef) -> Option<Self> {
    text.get(start..).and_then(|rest| {
      (!rest.is_empty()).then(|| Self {
        context: InputContext { text, rest },
        start,
        state,
        heap,
      })
    })
  }

  /// The whole input text.
  ///
  /// You can access the whole input text instead of only the rest of text,
  /// so that you can check chars before the [`Self::start`] position if needed.
  pub const fn text(&self) -> &'text str {
    self.context.text
  }

  /// The index of [`Self::text`], in bytes.
  ///
  /// This is guaranteed to be smaller than the length of [`Self::text`].
  pub const fn start(&self) -> usize {
    self.start
  }

  /// The undigested part of the input text.
  /// This is guaranteed to be not empty.
  ///
  /// This is precalculated in [`Self::new`] and cached to prevent creating the slice every time
  /// because this is frequently used across combinators.
  ///
  /// If you just want to get the next char, use [`Self::next`] instead.
  pub const fn rest(&self) -> &'text str {
    self.context.rest
  }

  /// The next char in the rest of the input text.
  pub fn next(&self) -> char {
    // SAFETY: `self.rest()` is guaranteed to be not empty.
    unsafe { self.rest().chars().next().unwrap_unchecked() }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn input_at_start() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new("123", 0, &mut state, &mut heap).unwrap();
    assert_eq!(input.text(), "123");
    assert_eq!(input.start(), 0);
    assert_eq!(input.rest(), "123");
    assert_eq!(input.next(), '1');
  }

  #[test]
  fn input_in_the_middle() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new("123", 1, &mut state, &mut heap).unwrap();
    assert_eq!(input.text(), "123");
    assert_eq!(input.start(), 1);
    assert_eq!(input.rest(), "23");
    assert_eq!(input.next(), '2');
  }

  #[test]
  fn input_no_rest() {
    assert!(Input::new("123", 3, &mut (), &mut ()).is_none());
  }

  #[test]
  fn input_out_of_text() {
    assert!(Input::new("123", 4, &mut (), &mut ()).is_none());
  }

  #[test]
  fn input_invalid_utf8_boundary() {
    assert!(Input::new("好", 1, &mut (), &mut ()).is_none());
  }
}
