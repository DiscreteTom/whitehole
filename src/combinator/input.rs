use super::Output;

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

  /// See [`Self::start`].
  start: usize,
  /// See [`Input::rest`].
  rest: &'text str,
}

impl<'text, StateRef, HeapRef> Input<'text, StateRef, HeapRef> {
  /// Return [`Some`] if `rest` is not empty.
  pub fn new(rest: &'text str, start: usize, state: StateRef, heap: HeapRef) -> Option<Self> {
    (!rest.is_empty()).then(|| Self {
      rest,
      start,
      state,
      heap,
    })
  }

  /// The index of the whole input text, in bytes.
  ///
  /// This will never be mutated after the creation of this instance.
  pub const fn start(&self) -> usize {
    self.start
  }

  /// The undigested part of the input text.
  /// This is guaranteed to be non-empty.
  ///
  /// If you just want to get the next char, use [`Self::next`] instead.
  pub const fn rest(&self) -> &'text str {
    self.rest
  }

  /// The first char in [`Self::rest`].
  ///
  /// Since [`Self::rest`] is guaranteed to be non-empty,
  /// the next char is guaranteed to be available.
  ///
  /// This is faster than `self.rest().chars().next().unwrap()`.
  pub fn next(&self) -> char {
    // SAFETY: `self.rest()` is guaranteed to be not empty.
    // TODO: make this faster by override `core::str::validations::next_code_point`
    unsafe { self.rest().chars().next().unwrap_unchecked() }
  }

  /// Try to build an [`Output`] by digesting `n` bytes.
  /// Return [`None`] if the [`Output::rest`] can't be built
  /// as a valid UTF-8 string.
  pub fn digest(&self, n: usize) -> Option<Output<'text, ()>> {
    self.rest.get(n..).map(|rest| Output { kind: (), rest })
  }

  /// Try to build an [`Output`] by digesting `n` bytes.
  /// # Safety
  /// You should ensure that [`Output::rest`] can be built
  /// as a valid UTF-8 string.
  /// This will be checked using [`debug_assert!`].
  /// For the checked version, see [`Self::digest`].
  pub unsafe fn digest_unchecked(&self, n: usize) -> Output<'text, ()> {
    debug_assert!(self.rest.get(n..).is_some());
    Output {
      kind: (),
      rest: self.rest.get_unchecked(n..),
    }
  }
}

// TODO: is this function's lifetime correct?
impl<'text, State, Heap> Input<'text, &mut State, &mut Heap> {
  /// Try to construct a new [`Input`] with the provided `rest`.
  /// The [`start`](Self::start) of the new instance will be auto calculated.
  ///
  /// Return [`Some`] if `rest` not empty.
  pub fn reload(&mut self, rest: &'text str) -> Option<Input<'text, &mut State, &mut Heap>> {
    Input::new(
      rest,
      self.rest.len() - rest.len() + self.start,
      &mut *self.state,
      &mut *self.heap,
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn input_new() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new("123", 0, &mut state, &mut heap).unwrap();
    assert_eq!(input.start(), 0);
    assert_eq!(input.rest(), "123");
    assert_eq!(input.next(), '1');
  }

  #[test]
  fn input_new_no_rest() {
    assert!(Input::new("", 0, &mut (), &mut ()).is_none());
  }

  #[test]
  fn input_reload() {
    let mut state = ();
    let mut heap = ();
    let mut input = Input::new("123", 0, &mut state, &mut heap).unwrap();
    assert_eq!(input.reload("23").unwrap().start(), 1);
    assert_eq!(input.reload("3").unwrap().start(), 2);
    assert!(input.reload("").is_none());
  }

  #[test]
  fn input_digest() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new("123", 0, &mut state, &mut heap).unwrap();
    assert_eq!(input.digest(3).map(|output| output.rest), Some(""));
    assert_eq!(input.digest(2).map(|output| output.rest), Some("3"));
    assert_eq!(input.digest(1).map(|output| output.rest), Some("23"));
    assert_eq!(input.digest(0).map(|output| output.rest), Some("123"));
    assert!(input.digest(4).is_none());
  }

  #[test]
  fn input_digest_unchecked() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new("123", 0, &mut state, &mut heap).unwrap();
    assert_eq!(unsafe { input.digest_unchecked(3).rest }, "");
    assert_eq!(unsafe { input.digest_unchecked(2).rest }, "3");
    assert_eq!(unsafe { input.digest_unchecked(1).rest }, "23");
    assert_eq!(unsafe { input.digest_unchecked(0).rest }, "123");
  }

  #[test]
  #[should_panic]
  fn input_digest_unchecked_overflow() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new("123", 0, &mut state, &mut heap).unwrap();
    unsafe { input.digest_unchecked(4) };
  }

  #[test]
  #[should_panic]
  fn input_digest_unchecked_invalid_code_point() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new("好", 0, &mut state, &mut heap).unwrap();
    unsafe { input.digest_unchecked(1) };
  }
}
