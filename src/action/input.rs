/// The input of [`Action::exec`](crate::action::Action::exec).
#[derive(Debug)]
pub struct Input<InstantRef, StateRef, HeapRef> {
  /// The `&Instant`.
  /// See [`Parser::instant`](crate::parser::Parser::instant).
  pub instant: InstantRef,

  /// The `&mut State`.
  /// See [`Parser::state`](crate::parser::Parser::state).
  pub state: StateRef,

  /// The `&mut Heap`.
  /// See [`Parser::heap`](crate::parser::Parser::heap).
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

  /// Re-borrow [`Self::state`] and [`Self::heap`] to construct a new instance
  /// with a new `Instant`.
  ///
  /// This is cheap to call.
  #[inline]
  pub const fn reborrow_with<'a>(
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
