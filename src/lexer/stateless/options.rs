use crate::lexer::{
  expectation::Expectation,
  fork::ForkEnabled,
  options::{LexOptions, TrimOptions},
  re_lex::ReLexContext,
};

/// Add [`Self::start`], [`Self::state`] and [`Self::heap`] to the `Base` options.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatelessOptions<StateRef, HeapRef, Base> {
  /// See [`Self::start`].
  pub start: usize,
  /// See [`Self::state`].
  pub state: StateRef,
  /// See [`Self::heap`].
  pub heap: HeapRef,
  pub base: Base,
}

impl<StateRef, HeapRef, Base> StatelessOptions<StateRef, HeapRef, Base> {
  /// The start index of the text to lex.
  #[inline]
  pub const fn start(mut self, start: usize) -> Self {
    self.start = start;
    self
  }

  /// Set the state.
  /// This is usually `&mut State`.
  /// For peek, this is `&State`.
  #[inline]
  pub fn state<NewStateRef>(
    self,
    state: NewStateRef,
  ) -> StatelessOptions<NewStateRef, HeapRef, Base> {
    StatelessOptions {
      start: self.start,
      state,
      heap: self.heap,
      base: self.base,
    }
  }

  /// Set the heap.
  /// This should be `&mut Heap`.
  #[inline]
  pub fn heap<NewHeapRef>(self, heap: NewHeapRef) -> StatelessOptions<StateRef, NewHeapRef, Base> {
    StatelessOptions {
      start: self.start,
      state: self.state,
      heap,
      base: self.base,
    }
  }
}

/// Add [`StatelessLexOptions::start`], [`StatelessLexOptions::state`]
/// and [`StatelessLexOptions::heap`] to [`LexOptions`].
pub type StatelessLexOptions<'literal, Kind, StateRef, HeapRef, Fork> =
  StatelessOptions<StateRef, HeapRef, LexOptions<'literal, Kind, Fork>>;

impl<'literal, Kind> StatelessLexOptions<'literal, Kind, (), (), ()> {
  /// Create a new instance with `0` as the start index, no state and no heap.
  #[inline]
  pub const fn new() -> Self {
    Self {
      start: 0,
      // use `()` as a placeholder, users should use `self.state` to set this
      state: (),
      // use `()` as a placeholder, users should use `self.heap` to set this
      heap: (),
      base: LexOptions::new(),
    }
  }
}

// re-export from `LexOptions`
// but with `StatelessLexOptions` as the return type
// instead of `LexOptions`
impl<'literal, Kind, StateRef, HeapRef, Fork>
  StatelessLexOptions<'literal, Kind, StateRef, HeapRef, Fork>
{
  /// See [`LexOptions::expect`].
  #[inline]
  pub fn expect(mut self, expectation: impl Into<Expectation<'literal, Kind>>) -> Self {
    self.base.expectation = expectation.into();
    self
  }
  /// See [`LexOptions::expect_with`].
  #[inline]
  pub fn expect_with(
    mut self,
    f: impl FnOnce(Expectation<'literal, Kind>) -> Expectation<'literal, Kind>,
  ) -> Self {
    self.base.expectation = f(Expectation::default());
    self
  }

  /// See [`LexOptions::fork`].
  #[inline]
  pub fn fork(self) -> StatelessLexOptions<'literal, Kind, StateRef, HeapRef, ForkEnabled> {
    StatelessLexOptions {
      start: self.start,
      state: self.state,
      heap: self.heap,
      base: self.base.fork(),
    }
  }
  /// See [`LexOptions::re_lex`].
  #[inline]
  pub const fn re_lex(mut self, re_lex: ReLexContext) -> Self {
    self.base.re_lex = re_lex;
    self
  }
}

/// Add [`StatelessTrimOptions::start`], [`StatelessTrimOptions::state`]
/// and [`StatelessTrimOptions::heap`] to [`TrimOptions`].
pub type StatelessTrimOptions<StateRef, Heap> = StatelessOptions<StateRef, Heap, TrimOptions>;

impl StatelessTrimOptions<(), ()> {
  /// Create a new instance with `0` as the start index, no state and no heap.
  #[inline]
  pub const fn new() -> Self {
    Self {
      start: 0,
      // use `()` as a placeholder, user should use `self.state` to set this
      state: (),
      // use `()` as a placeholder, user should use `self.heap` to set this
      heap: (),
      base: TrimOptions,
    }
  }
}

impl Default for StatelessTrimOptions<(), ()> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! assert_unit {
    ($e: expr) => {
      let _: () = $e;
    };
  }

  #[test]
  fn test_stateless_options() {
    let options = StatelessOptions {
      start: 0,
      state: (),
      heap: (),
      base: (),
    };
    assert_eq!(options.start, 0);
    assert_unit!(options.state);
    assert_unit!(options.base);

    let options = options.start(1);
    assert_eq!(options.start, 1);
    assert_unit!(options.state);
    assert_unit!(options.base);

    let options = options.state(1);
    assert_eq!(options.start, 1);
    assert_eq!(options.state, 1);
    assert_unit!(options.base);
  }

  #[test]
  fn test_stateless_lex_options() {
    let options = StatelessLexOptions::new();
    assert_eq!(options.start, 0);
    assert_unit!(options.state);
    assert_eq!(options.base, LexOptions::<(), _>::new());

    let options = options.expect(Expectation::default());
    assert_eq!(options.start, 0);
    assert_unit!(options.state);
    assert_eq!(
      options.base,
      LexOptions::new().expect(Expectation::default())
    );

    let options = options.expect_with(|e| e.literal("a"));
    assert_eq!(options.start, 0);
    assert_unit!(options.state);
    assert_eq!(
      options.base,
      LexOptions::new().expect_with(|e| e.literal("a"))
    );

    let options: StatelessOptions<(), (), LexOptions<(), ForkEnabled>> =
      StatelessLexOptions::new().fork();
    assert_eq!(options.start, 0);
    assert_unit!(options.state);
    assert_eq!(options.base, LexOptions::new().fork());

    let options = options.re_lex(ReLexContext { start: 1, skip: 1 });
    assert_eq!(options.start, 0);
    assert_unit!(options.state);
    assert_eq!(
      options.base,
      LexOptions::new()
        .fork()
        .re_lex(ReLexContext { start: 1, skip: 1 })
    );
  }

  #[test]
  fn test_stateless_trim_options() {
    let options = StatelessTrimOptions::new();
    assert_eq!(options.start, 0);
    assert_unit!(options.state);
    assert_eq!(options.base, TrimOptions);
  }
}
