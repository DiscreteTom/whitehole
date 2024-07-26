use crate::lexer::{
  expectation::Expectation,
  fork::{ForkDisabled, ForkEnabled},
  options::{LexOptions, TrimOptions},
  re_lex::ReLexContext,
};

/// Add [`Self::start`] and [`Self::action_state`] to the `Base` options.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatelessOptions<ActionStateRef, Base> {
  /// See [`Self::start`].
  pub start: usize,
  /// See [`Self::action_state`].
  pub action_state: ActionStateRef,
  pub base: Base,
}

impl<ActionStateRef, Base> StatelessOptions<ActionStateRef, Base> {
  /// The start index of the text to lex.
  #[inline]
  pub const fn start(mut self, start: usize) -> Self {
    self.start = start;
    self
  }

  /// Set the action state.
  /// This is usually `&mut ActionState`.
  #[inline]
  pub fn action_state<NewActionStateRef>(
    self,
    action_state: NewActionStateRef,
  ) -> StatelessOptions<NewActionStateRef, Base> {
    StatelessOptions {
      start: self.start,
      action_state,
      base: self.base,
    }
  }
}

/// Add [`StatelessLexOptions::start`] and [`StatelessLexOptions::action_state`] to [`LexOptions`].
pub type StatelessLexOptions<'expect_literal, Kind, ActionStateRef, ErrAcc, Fork> =
  StatelessOptions<ActionStateRef, LexOptions<'expect_literal, Kind, ErrAcc, Fork>>;

impl<'expect_literal, Kind> StatelessLexOptions<'expect_literal, Kind, (), (), ForkDisabled> {
  /// Create a new instance with `0` as the start index and no action state.
  #[inline]
  pub const fn new() -> Self {
    Self {
      start: 0,
      // use `()` as a placeholder, users should use `self.action_state` to set this
      action_state: (),
      base: LexOptions::new(),
    }
  }
}

// re-export from `LexOptions`
// but with `StatelessLexOptions` as the return type
// instead of `LexOptions`
impl<'expect_literal, Kind, ActionStateRef, ErrAcc, Fork>
  StatelessLexOptions<'expect_literal, Kind, ActionStateRef, ErrAcc, Fork>
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
  /// See [`LexOptions::errors_to`].
  #[inline]
  pub fn errors_to<NewErrAcc>(
    self,
    acc: NewErrAcc,
  ) -> StatelessLexOptions<'expect_literal, Kind, ActionStateRef, NewErrAcc, Fork> {
    StatelessLexOptions {
      start: self.start,
      action_state: self.action_state,
      base: self.base.errors_to(acc),
    }
  }
  /// See [`LexOptions::errors_to_vec`].
  #[inline]
  pub fn errors_to_vec(
    self,
  ) -> StatelessLexOptions<'expect_literal, Kind, ActionStateRef, Vec<ErrAcc>, Fork> {
    StatelessLexOptions {
      start: self.start,
      action_state: self.action_state,
      base: self.base.errors_to_vec(),
    }
  }
  /// See [`LexOptions::fork`].
  #[inline]
  pub fn fork(
    self,
  ) -> StatelessLexOptions<'expect_literal, Kind, ActionStateRef, ErrAcc, ForkEnabled> {
    StatelessLexOptions {
      start: self.start,
      action_state: self.action_state,
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

/// Add [`StatelessTrimOptions::start`] and [`StatelessTrimOptions::action_state`] to [`TrimOptions`].
pub type StatelessTrimOptions<ActionStateRef, ErrAcc> =
  StatelessOptions<ActionStateRef, TrimOptions<ErrAcc>>;

impl StatelessTrimOptions<(), ()> {
  /// Create a new instance with `0` as the start index and no action state.
  #[inline]
  pub const fn new() -> Self {
    Self {
      start: 0,
      // use `()` as a placeholder, user should use `self.action_state` to set this
      action_state: (),
      base: TrimOptions::new(),
    }
  }
}

// re-export from `TrimOptions`
// but with `StatelessTrimOptions` as the return type
// instead of `TrimOptions`
impl<ActionStateRef, ErrAcc> StatelessTrimOptions<ActionStateRef, ErrAcc> {
  /// See [`TrimOptions::errors_to`].
  #[inline]
  pub fn errors_to<NewErrAcc>(
    self,
    acc: NewErrAcc,
  ) -> StatelessTrimOptions<ActionStateRef, NewErrAcc> {
    StatelessTrimOptions {
      start: self.start,
      action_state: self.action_state,
      base: self.base.errors_to(acc),
    }
  }

  /// See [`TrimOptions::errors_to_vec`].
  #[inline]
  pub fn errors_to_vec(self) -> StatelessTrimOptions<ActionStateRef, Vec<ErrAcc>> {
    StatelessTrimOptions {
      start: self.start,
      action_state: self.action_state,
      base: self.base.errors_to_vec(),
    }
  }
}
