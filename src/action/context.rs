// TODO: is this struct needed? can we just pass `&mut state` and `&mut heap` directly?

/// This provides `&mut State` and `&mut Heap`.
#[derive(Debug)]
pub struct Context<StateRef, HeapRef> {
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

impl Default for Context<&mut (), &mut ()> {
  fn default() -> Self {
    static mut UNIT: () = ();
    Context {
      state: unsafe { &mut UNIT },
      heap: unsafe { &mut UNIT },
    }
  }
}

impl<State, Heap> Context<&mut State, &mut Heap> {
  /// Re-borrow [`Self::state`] and [`Self::heap`] to construct a new instance
  /// (similar to cloning this instance).
  ///
  /// This is cheap to call.
  #[inline]
  pub fn reborrow(&mut self) -> Context<&mut State, &mut Heap> {
    Context {
      state: &mut *self.state,
      heap: &mut *self.heap,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn context_reborrow() {
    let mut state = 123;
    let mut heap = 123;
    let mut ctx = Context {
      state: &mut state,
      heap: &mut heap,
    };
    {
      let ctx = ctx.reborrow();
      *ctx.state = 456;
      *ctx.heap = 456;
    }
    assert_eq!(state, 456);
    assert_eq!(heap, 456);
  }
}
