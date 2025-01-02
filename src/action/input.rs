use crate::instant::Instant;

/// The input of [`Action::exec`](crate::action::Action::exec).
/// `self.instant().rest()` is guaranteed to be non-empty.
///
/// Once created, only [`Self::state`] and [`Self::heap`] can be mutated.
///
/// If you want to clone this, see [`Self::reborrow`].
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

  /// See [`Self::instant`].
  instant: Instant<'text>,
}

impl<'text, StateRef, HeapRef> Input<'text, StateRef, HeapRef> {
  /// # Safety
  /// You should ensure that [`Instant::rest`] is not empty.
  /// This will be checked using [`debug_assert!`].
  /// For the checked version, see [`Self::new`].
  #[inline]
  pub const unsafe fn new_unchecked(
    instant: Instant<'text>,
    state: StateRef,
    heap: HeapRef,
  ) -> Self {
    debug_assert!(!instant.rest().is_empty());
    Self {
      instant,
      state,
      heap,
    }
  }

  /// Return [`Some`] if [`Instant::rest`] is not empty.
  #[inline]
  pub fn new(instant: Instant<'text>, state: StateRef, heap: HeapRef) -> Option<Self> {
    (!instant.rest().is_empty()).then(|| unsafe { Self::new_unchecked(instant, state, heap) })
  }

  /// The [`Instant`] before this action is executed.
  /// [`Instant::rest`] is guaranteed to be non-empty.
  #[inline]
  pub const fn instant(&self) -> &Instant<'text> {
    &self.instant
  }

  /// The first char in [`Instant::rest`].
  ///
  /// Since `self.instant().rest()` is guaranteed to be non-empty,
  /// the next char is guaranteed to exist.
  ///
  /// This is faster than `self.instant().rest().chars().next().unwrap()`.
  ///
  /// This value is not stored in this struct
  /// because the value is not always needed.
  /// You can cache the return value as needed.
  #[inline]
  pub fn next(&self) -> char {
    // SAFETY: `self.instant.rest()` is guaranteed to be not empty.
    unsafe { self.instant.rest().chars().next().unwrap_unchecked() }
  }
}

impl<'text, State, Heap> Input<'text, &mut State, &mut Heap> {
  /// Re-borrow [`Self::state`] and [`Self::heap`] to construct a new [`Input`]
  /// (similar to cloning this instance).
  ///
  /// This is cheap to call.
  #[inline]
  pub fn reborrow(&mut self) -> Input<'text, &mut State, &mut Heap> {
    Input {
      state: &mut *self.state,
      heap: &mut *self.heap,
      instant: self.instant.clone(),
    }
  }
}
/// Construct a new [`Input`] by digesting `n` bytes from [`Input::instant`].
///
/// Return [`Some`] if [`Instant::rest`] of the new instant is not empty.
/// # Safety
/// You should ensure that `n` is a valid UTF-8 boundary.
/// This will be checked using [`debug_assert!`].
/// # Performance
/// This is a macro to make sure this is always inlined.
macro_rules! shift_input {
  ($input:expr, $n:expr) => {
    // perf: check the len first to prevent unnecessary clone of instant
    ($n < $input.instant().rest().len()).then(|| {
      let mut instant = $input.instant().clone();
      unsafe {
        instant.digest_unchecked($n);
        Input::new_unchecked(instant, &mut *$input.state, &mut *$input.heap)
      }
    })
  };
}
pub(crate) use shift_input;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn input_new_unchecked() {
    let mut state = ();
    let mut heap = ();
    let input = unsafe { Input::new_unchecked(Instant::new("123"), &mut state, &mut heap) };
    assert_eq!(input.instant().text(), "123");
    assert_eq!(input.instant().rest(), "123");
    assert_eq!(input.instant().digested(), 0);
    assert_eq!(input.next(), '1');
  }

  #[test]
  #[should_panic]
  fn input_new_unchecked_empty() {
    unsafe { Input::new_unchecked(Instant::new(""), &mut (), &mut ()) };
  }

  #[test]
  fn input_new() {
    let mut state = ();
    let mut heap = ();
    let input = Input::new(Instant::new("123"), &mut state, &mut heap).unwrap();
    assert_eq!(input.instant().digested(), 0);
    assert_eq!(input.instant().rest(), "123");
    assert_eq!(input.next(), '1');
  }

  #[test]
  fn input_new_no_rest() {
    assert!(Input::new(Instant::new(""), &mut (), &mut ()).is_none());
  }

  #[test]
  fn input_reborrow() {
    let mut state = 123;
    let mut heap = 123;
    let mut input = Input::new(Instant::new("123"), &mut state, &mut heap).unwrap();
    {
      let input = input.reborrow();
      assert_eq!(input.instant().digested(), 0);
      assert_eq!(input.instant().rest(), "123");
      assert_eq!(input.next(), '1');
      *input.state = 456;
      *input.heap = 456;
    }
    assert_eq!(state, 456);
    assert_eq!(heap, 456);
  }
}
