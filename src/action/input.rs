/// The input of [`Action::exec`](crate::action::Action::exec).
///
/// Once created, only [`Self::state`] and [`Self::heap`] can be mutated.
///
/// For simplicity, there is no `Input::text` to get the whole input text,
/// you can only use [`Input::rest`] to get the undigested part.
/// If you do need the whole input text, you can store it in [`Self::heap`].
#[derive(Debug)]
pub struct Input<'text, StateRef, HeapRef> {
  /// The `&mut State`.
  /// This is public, so you can mutate the `State` directly.
  ///
  /// With the `State`, you can construct stateful parsers,
  /// while actions remain stateless and clone-able.
  ///
  /// All vars that control the flow of the parsing should be stored here.
  /// This should be small and cheap to clone (maybe just a bunch of integers or booleans).
  /// If a var only represents a resource (e.g. a chunk of memory, a channel, etc),
  /// it should be stored in [`Self::heap`].
  pub state: StateRef,
  /// The `&mut Heap`.
  /// This is public, so you can mutate this directly.
  ///
  /// With the `Heap`, you can re-use allocated memory
  /// across actions and parsings.
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
  /// # Safety
  /// You should ensure that `rest` is not empty.
  /// This will be checked using [`debug_assert!`].
  /// For the checked version, see [`Self::new`].
  #[inline]
  pub const unsafe fn new_unchecked(
    rest: &'text str,
    start: usize,
    state: StateRef,
    heap: HeapRef,
  ) -> Self {
    debug_assert!(!rest.is_empty());
    Self {
      rest,
      start,
      state,
      heap,
    }
  }

  /// Return [`Some`] if `rest` is not empty.
  #[inline]
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
  /// This is cheap to call because the value is stored in this struct.
  /// This will never be mutated after the creation of this instance.
  #[inline]
  pub const fn start(&self) -> usize {
    self.start
  }

  /// The undigested part of the input text.
  /// This is guaranteed to be non-empty.
  ///
  /// This is cheap to call because the value is stored in this struct.
  /// This will never be mutated after the creation of this instance.
  ///
  /// If you just want to get the next char, use [`Self::next`] instead.
  #[inline]
  pub const fn rest(&self) -> &'text str {
    self.rest
  }

  /// The first char in [`Self::rest`].
  ///
  /// Since [`Self::rest`] is guaranteed to be non-empty,
  /// the next char is guaranteed to be available.
  ///
  /// This is faster than `self.rest().chars().next().unwrap()`.
  ///
  /// This value is not stored in this struct
  /// because the value is not always needed.
  /// You can cache the return value as needed.
  #[inline]
  pub fn next(&self) -> char {
    // SAFETY: `self.rest()` is guaranteed to be not empty.
    unsafe { self.rest().chars().next().unwrap_unchecked() }
  }
}

impl<'text, State, Heap> Input<'text, &mut State, &mut Heap> {
  /// Re-borrow [`Self::state`] and [`Self::heap`] to construct a new [`Input`]
  /// (similar to cloning this instance).
  ///
  /// This is cheap to call.
  #[inline]
  pub const fn reborrow(&mut self) -> Input<'text, &mut State, &mut Heap> {
    Input {
      state: &mut *self.state,
      heap: &mut *self.heap,
      start: self.start,
      rest: self.rest,
    }
  }

  /// Construct a new [`Input`] by shifting [`Self::start`] forward by `n` bytes.
  /// [`Self::rest`] of the new instance will be auto calculated.
  /// # Safety
  /// You should ensure that `n` is a valid UTF-8 boundary
  /// and smaller than the length of [`Self::rest`].
  /// This will be checked using [`debug_assert!`].
  /// For the checked version, see [`Self::shift`].
  #[inline]
  pub unsafe fn shift_unchecked(&mut self, n: usize) -> Input<'text, &mut State, &mut Heap> {
    debug_assert!(self.rest.is_char_boundary(n));
    debug_assert!(n < self.rest.len());
    Input::new_unchecked(
      self.rest.get_unchecked(n..),
      self.start.unchecked_add(n),
      &mut *self.state,
      &mut *self.heap,
    )
  }

  /// Try to construct a new [`Input`] by shifting [`Self::start`] forward by `n` bytes.
  /// [`Self::rest`] of the new instance will be auto calculated.
  ///
  /// Return [`Some`] if `n` is a valid UTF-8 boundary
  /// and smaller than the length of [`Self::rest`].
  #[inline]
  pub fn shift(&mut self, n: usize) -> Option<Input<'text, &mut State, &mut Heap>> {
    if n >= self.rest.len() {
      return None;
    }

    self
      .rest
      .get(n..)
      .and_then(|_| unsafe { self.shift_unchecked(n) }.into())
    // TODO: optimize code?
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn input_new_unchecked() {
    let _ = unsafe { Input::new_unchecked("123", 0, &mut (), &mut ()) };
  }

  #[test]
  #[should_panic]
  fn input_new_unchecked_empty() {
    let _ = unsafe { Input::new_unchecked("", 0, &mut (), &mut ()) };
  }

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
  fn input_shift() {
    let mut state = ();
    let mut heap = ();
    let mut input = Input::new("123", 0, &mut state, &mut heap).unwrap();
    assert_eq!(input.shift(1).unwrap().start(), 1);
    assert_eq!(input.shift(2).unwrap().start(), 2);
    assert!(input.shift(3).is_none());
  }

  // TODO: add tests for shift_unchecked
}
