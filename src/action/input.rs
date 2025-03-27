/// The input of [`Action::exec`](crate::action::Action::exec).
pub struct Input<InstantRef, StateRef, HeapRef> {
  pub instant: InstantRef,
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
}

impl<Instant, State, Heap> Input<&Instant, &mut State, &mut Heap> {
  /// Re-borrow [`Self::state`] and [`Self::heap`] to construct a new instance
  /// (similar to cloning this instance).
  ///
  /// This is cheap to call.
  #[inline]
  pub const fn reborrow(&mut self) -> Input<&Instant, &mut State, &mut Heap> {
    Input {
      instant: self.instant,
      state: self.state,
      heap: self.heap,
    }
  }

  // TODO: better name
  /// Re-borrow [`Self::state`] and [`Self::heap`] to construct a new instance
  /// with a new `Instant`.
  ///
  /// This is cheap to call.
  #[inline]
  pub const fn reload<'a>(
    &mut self,
    instant: &'a Instant,
  ) -> Input<&'a Instant, &mut State, &mut Heap> {
    Input {
      instant,
      state: self.state,
      heap: self.heap,
    }
  }
}
