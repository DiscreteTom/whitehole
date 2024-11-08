#[derive(Debug)]
pub struct Input<Context, StateRef, HeapRef> {
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

  /// This should not be mutated by users directly, thus not public.
  pub(crate) start: usize,
  /// This should not be accessed by users directly, thus not public.
  pub(crate) context: Context,
}
