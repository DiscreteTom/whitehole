use crate::{
  lexer::{
    expectation::Expectation,
    fork::ForkEnabled,
    options::{LexOptions, TrimOptions},
    re_lex::ReLexContext,
  },
  utils::AccumulatorSetter,
};

/// Add [`Self::start`] and [`Self::state`] to the `Base` options.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatelessOptions<StateRef, Base> {
  /// See [`Self::start`].
  pub start: usize,
  /// See [`Self::state`].
  pub state: StateRef,
  pub base: Base,
}

impl<StateRef, Base> StatelessOptions<StateRef, Base> {
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
  pub fn state<NewStateRef>(self, state: NewStateRef) -> StatelessOptions<NewStateRef, Base> {
    StatelessOptions {
      start: self.start,
      state,
      base: self.base,
    }
  }
}

/// Add [`StatelessLexOptions::start`] and [`StatelessLexOptions::state`] to [`LexOptions`].
pub type StatelessLexOptions<'expect_literal, Kind, StateRef, ErrAcc, Fork> =
  StatelessOptions<StateRef, LexOptions<'expect_literal, Kind, ErrAcc, Fork>>;

impl<'expect_literal, Kind> StatelessLexOptions<'expect_literal, Kind, (), (), ()> {
  /// Create a new instance with `0` as the start index and no state.
  #[inline]
  pub const fn new() -> Self {
    Self {
      start: 0,
      // use `()` as a placeholder, users should use `self.state` to set this
      state: (),
      base: LexOptions::new(),
    }
  }
}

// re-export from `LexOptions`
// but with `StatelessLexOptions` as the return type
// instead of `LexOptions`
impl<'expect_literal, Kind, StateRef, ErrAcc, Fork>
  StatelessLexOptions<'expect_literal, Kind, StateRef, ErrAcc, Fork>
{
  /// See [`LexOptions::expect`].
  #[inline]
  pub fn expect(mut self, expectation: impl Into<Expectation<'expect_literal, Kind>>) -> Self {
    self.base.expectation = expectation.into();
    self
  }
  /// See [`LexOptions::expect_with`].
  #[inline]
  pub fn expect_with(
    mut self,
    f: impl FnOnce(Expectation<'expect_literal, Kind>) -> Expectation<'expect_literal, Kind>,
  ) -> Self {
    self.base.expectation = f(Expectation::default());
    self
  }
  /// See [`LexOptions::errors`].
  #[inline]
  pub fn errors<NewErrAcc>(
    self,
  ) -> AccumulatorSetter<
    impl FnOnce(NewErrAcc) -> StatelessLexOptions<'expect_literal, Kind, StateRef, NewErrAcc, Fork>,
  > {
    AccumulatorSetter::new(move |acc| StatelessLexOptions {
      start: self.start,
      state: self.state,
      base: self.base.errors().to(acc),
    })
  }
  /// See [`LexOptions::fork`].
  #[inline]
  pub fn fork(self) -> StatelessLexOptions<'expect_literal, Kind, StateRef, ErrAcc, ForkEnabled> {
    StatelessLexOptions {
      start: self.start,
      state: self.state,
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

/// Add [`StatelessTrimOptions::start`] and [`StatelessTrimOptions::state`] to [`TrimOptions`].
pub type StatelessTrimOptions<StateRef, ErrAcc> = StatelessOptions<StateRef, TrimOptions<ErrAcc>>;

impl StatelessTrimOptions<(), ()> {
  /// Create a new instance with `0` as the start index and no state.
  #[inline]
  pub const fn new() -> Self {
    Self {
      start: 0,
      // use `()` as a placeholder, user should use `self.state` to set this
      state: (),
      base: TrimOptions::new(),
    }
  }
}

// re-export from `TrimOptions`
// but with `StatelessTrimOptions` as the return type
// instead of `TrimOptions`
impl<StateRef, ErrAcc> StatelessTrimOptions<StateRef, ErrAcc> {
  /// See [`TrimOptions::errors`].
  #[inline]
  pub fn errors<NewErrAcc>(
    self,
  ) -> AccumulatorSetter<impl FnOnce(NewErrAcc) -> StatelessTrimOptions<StateRef, NewErrAcc>> {
    AccumulatorSetter::new(move |acc| StatelessTrimOptions {
      start: self.start,
      state: self.state,
      base: self.base.errors().to(acc),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_stateless_options() {
    let options = StatelessOptions {
      start: 0,
      state: (),
      base: (),
    };
    assert_eq!(options.start, 0);
    assert_eq!(options.state, ());
    assert_eq!(options.base, ());

    let options = options.start(1);
    assert_eq!(options.start, 1);
    assert_eq!(options.state, ());
    assert_eq!(options.base, ());

    let options = options.state(1);
    assert_eq!(options.start, 1);
    assert_eq!(options.state, 1);
    assert_eq!(options.base, ());
  }

  #[test]
  fn test_stateless_lex_options() {
    let options = StatelessLexOptions::new();
    assert_eq!(options.start, 0);
    assert_eq!(options.state, ());
    assert_eq!(options.base, LexOptions::<(), _, _>::new());

    let options = options.expect(Expectation::default());
    assert_eq!(options.start, 0);
    assert_eq!(options.state, ());
    assert_eq!(
      options.base,
      LexOptions::new().expect(Expectation::default())
    );

    let options = options.expect_with(|e| e.literal("a"));
    assert_eq!(options.start, 0);
    assert_eq!(options.state, ());
    assert_eq!(
      options.base,
      LexOptions::new().expect_with(|e| e.literal("a"))
    );

    let mut errors: Vec<()> = vec![];
    let options = options.expect_with(|e| e).errors().to(&mut errors);
    assert_eq!(options.start, 0);
    assert_eq!(options.state, ());
    assert_eq!(options.base, LexOptions::new().errors().to(&mut vec![]));

    let options = options.errors().to(()).fork();
    assert_eq!(options.start, 0);
    assert_eq!(options.state, ());
    assert_eq!(options.base, LexOptions::new().fork());

    let options = options.re_lex(ReLexContext { start: 1, skip: 1 });
    assert_eq!(options.start, 0);
    assert_eq!(options.state, ());
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
    assert_eq!(options.state, ());
    assert_eq!(options.base, TrimOptions::new());

    let mut errors: Vec<()> = vec![];
    let options = StatelessTrimOptions::new().errors().to(&mut errors);
    assert_eq!(options.start, 0);
    assert_eq!(options.state, ());
    assert_eq!(options.base, TrimOptions::new().errors().to(&mut vec![]));

    let options = StatelessTrimOptions::new().errors().to(&mut errors);
    assert_eq!(options.start, 0);
    assert_eq!(options.state, ());
    assert_eq!(options.base, TrimOptions::new().errors().to(&mut vec![]));
  }
}
