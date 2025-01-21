use crate::instant::Instant;

/// The input of [`Action::exec`](crate::action::Action::exec).
///
/// Once created, only [`Self::state`] and [`Self::heap`] can be mutated.
///
/// If you want to clone this, see [`Self::reborrow`].
#[derive(Debug)]
pub struct Input<TextRef, StateRef, HeapRef> {
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

  /// See [`Self::instant`].
  instant: Instant<TextRef>,
}

impl<TextRef, StateRef, HeapRef> Input<TextRef, StateRef, HeapRef> {
  #[inline]
  pub const fn new(instant: Instant<TextRef>, state: StateRef, heap: HeapRef) -> Self {
    Input {
      state,
      heap,
      instant,
    }
  }

  /// The [`Instant`] before this action is executed.
  #[inline]
  pub const fn instant(&self) -> &Instant<TextRef> {
    &self.instant
  }
}

impl<TextRef: Clone, State, Heap> Input<TextRef, &mut State, &mut Heap> {
  /// Re-borrow [`Self::state`] and [`Self::heap`] to construct a new [`Input`]
  /// (similar to cloning this instance).
  ///
  /// This is cheap to call.
  #[inline]
  pub fn reborrow(&mut self) -> Input<TextRef, &mut State, &mut Heap> {
    Input {
      state: &mut *self.state,
      heap: &mut *self.heap,
      instant: self.instant.clone(),
    }
  }
}

impl<'text, State, Heap> Input<&'text [u8], &mut State, &mut Heap> {
  /// Construct a new [`Input`] by digesting `n` bytes from [`Input::instant`].
  ///
  /// This is cheap to call.
  /// # Safety
  /// You should ensure that `n` is smaller than the length of [`Instant::rest`].
  /// This will be checked using [`debug_assert!`].
  #[inline]
  pub unsafe fn shift_unchecked(&mut self, n: usize) -> Input<&'text [u8], &mut State, &mut Heap> {
    let mut instant = self.instant.clone();
    instant.digest_unchecked(n);
    Input::new(instant, &mut *self.state, &mut *self.heap)
  }
}

impl<'text, State, Heap> Input<&'text str, &mut State, &mut Heap> {
  /// Construct a new [`Input`] by digesting `n` bytes from [`Input::instant`].
  ///
  /// This is cheap to call.
  /// # Safety
  /// You should ensure that `n` is a valid UTF-8 boundary.
  /// This will be checked using [`debug_assert!`].
  #[inline]
  pub unsafe fn shift_unchecked(&mut self, n: usize) -> Input<&'text str, &mut State, &mut Heap> {
    // TODO: simplify code with a trait?
    let mut instant = self.instant.clone();
    instant.digest_unchecked(n);
    Input::new(instant, &mut *self.state, &mut *self.heap)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn input_new_getters() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new(Instant::new("123"), &mut state, &mut heap);
    assert_eq!(input.instant().digested(), 0);
    assert_eq!(input.instant().rest(), "123");
  }

  #[test]
  fn input_new_no_rest_is_ok() {
    Input::new(Instant::new(""), &mut (), &mut ());
  }

  #[test]
  fn input_reborrow() {
    let mut state = 123;
    let mut heap = 123;
    let mut input = Input::new(Instant::new("123"), &mut state, &mut heap);
    {
      let input = input.reborrow();
      assert_eq!(input.instant().digested(), 0);
      assert_eq!(input.instant().rest(), "123");
      *input.state = 456;
      *input.heap = 456;
    }
    assert_eq!(state, 456);
    assert_eq!(heap, 456);
  }

  #[test]
  fn input_shift_unchecked() {
    let mut state = 123;
    let mut heap = 123;
    let mut input = Input::new(Instant::new("123"), &mut state, &mut heap);
    let input = unsafe { input.shift_unchecked(1) };
    assert_eq!(input.instant().digested(), 1);
    assert_eq!(input.instant().rest(), "23");

    let mut state = 123;
    let mut heap = 123;
    let mut input = Input::new(Instant::new(&[1u8, 2, 3] as &[u8]), &mut state, &mut heap);
    let input = unsafe { input.shift_unchecked(1) };
    assert_eq!(input.instant().digested(), 1);
    assert_eq!(input.instant().rest(), &[2u8, 3] as &[u8]);
  }

  #[test]
  #[should_panic]
  fn input_shift_unchecked_invalid_utf8() {
    let mut state = 123;
    let mut heap = 123;
    let mut input = Input::new(Instant::new("好"), &mut state, &mut heap);
    let _ = unsafe { input.shift_unchecked(1) };
  }

  #[test]
  #[should_panic]
  fn input_bytes_shift_overflow() {
    let mut state = 123;
    let mut heap = 123;
    let mut input = Input::new(Instant::new(&[1u8, 2, 3] as &[u8]), &mut state, &mut heap);
    let _ = unsafe { input.shift_unchecked(4) };
  }

  #[test]
  #[should_panic]
  fn input_str_shift_overflow() {
    let mut state = 123;
    let mut heap = 123;
    let mut input = Input::new(Instant::new("123"), &mut state, &mut heap);
    let _ = unsafe { input.shift_unchecked(4) };
  }
}
