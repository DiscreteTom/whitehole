/// [`Combinator`](crate::combinator::Combinator)'s input.
///
/// Once created, only [`Self::state`] and [`Self::heap`] can be mutated.
#[derive(Debug)]
pub struct Input<'text, StateRef, HeapRef> {
  /// The `&mut State`.
  ///
  /// This is public, so you can mutate the `State` directly.
  ///
  /// With the `State`, you can construct stateful parsers,
  /// while combinators remain stateless and clone-able.
  ///
  /// All vars that control the flow of the parsing should be stored here.
  /// This should be small and cheap to clone (maybe just a bunch of integers or booleans).
  /// If a var only represents a resource (e.g. a chunk of memory, a channel, etc),
  /// it should be stored in [`Self::heap`].
  pub state: StateRef,
  /// The `&mut Heap`.
  ///
  /// This is public, so you can mutate this directly.
  ///
  /// With the `Heap`, you can re-use allocated memory
  /// across combinator and parsings.
  ///
  /// All vars that doesn't count as a part of [`Self::state`] should be stored here.
  /// If a var is used to control the flow of the parsing,
  /// it should be treated as a state and stored in [`Self::state`].
  /// If a var only represents a resource (e.g. a chunk of memory, a channel, etc),
  /// it should be stored here.
  pub heap: HeapRef,

  /// See [`Input::text`].
  text: &'text str,
  /// See [`Self::start`].
  start: usize,
  /// See [`Input::rest`].
  rest: &'text str,
}

impl<'text, StateRef, HeapRef> Input<'text, StateRef, HeapRef> {
  /// Return [`Some`] if [`Self::rest`] can be constructed and not empty.
  pub fn new(text: &'text str, start: usize, state: StateRef, heap: HeapRef) -> Option<Self> {
    text.get(start..).and_then(|rest| {
      (!rest.is_empty()).then(|| Self {
        text,
        rest,
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
    self.text
  }

  /// The index of [`Self::text`], in bytes.
  ///
  /// This is guaranteed to be smaller than the length of [`Self::text`],
  /// and will never be mutated after the creation of this instance.
  pub const fn start(&self) -> usize {
    self.start
  }

  /// The undigested part of the input text.
  /// This is guaranteed to be non-empty.
  ///
  /// This is precalculated in [`Self::new`] and cached to prevent creating the slice every time
  /// because this is frequently used across combinators.
  ///
  /// If you just want to get the next char, use [`Self::next`] instead.
  pub const fn rest(&self) -> &'text str {
    self.rest
  }

  /// The next char in the rest of the input text.
  ///
  /// Since [`Self::rest`] is guaranteed to be non-empty,
  /// the next char is guaranteed to be available.
  pub fn next(&self) -> char {
    // SAFETY: `self.rest()` is guaranteed to be not empty.
    unsafe { self.rest().chars().next().unwrap_unchecked() }
  }
}

// TODO: is this function's lifetime correct?
impl<'text, State, Heap> Input<'text, &mut State, &mut Heap> {
  /// Try to construct a new [`Input`] by moving the [`Self::start`] forward by `n`.
  ///
  /// Return [`Some`] if [`Self::rest`] can be constructed and not empty.
  pub fn digest(&mut self, n: usize) -> Option<Input<'text, &mut State, &mut Heap>> {
    Input::new(self.text, self.start + n, &mut *self.state, &mut *self.heap)
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

  #[test]
  fn input_digest() {
    let mut state = ();
    let mut heap = ();
    let mut input = Input::new("123", 0, &mut state, &mut heap).unwrap();
    assert_eq!(input.digest(1).unwrap().rest(), "23");
    assert_eq!(input.digest(2).unwrap().rest(), "3");
    assert!(input.digest(3).is_none());
  }
}
