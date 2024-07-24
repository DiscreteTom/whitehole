use crate::lexer::{
  expectation::Expectation,
  fork::{ForkDisabled, ForkEnabled},
  options::LexOptions,
  re_lex::ReLexContext,
};

pub struct StatelessLexOptions<'expect_literal, Kind: 'static, ActionStateRef, ErrAcc, Fork> {
  /// See [`StatelessLexOptions::start()`].
  pub start: usize,
  /// This is usually `&mut ActionState`.
  pub action_state: ActionStateRef,
  pub err_acc: ErrAcc,
  // pub error_handler:
  pub base: LexOptions<'expect_literal, Kind, Fork>,
}

impl<'expect_literal, Kind> StatelessLexOptions<'expect_literal, Kind, (), (), ForkDisabled> {
  pub fn new() -> Self {
    Self {
      start: 0,
      action_state: (), // use `()` as a placeholder, user should use `self.action_state` to set this
      err_acc: (),
      base: LexOptions::default(),
    }
  }
}

impl<'expect_literal, Kind> From<Expectation<'expect_literal, Kind>>
  for StatelessLexOptions<'expect_literal, Kind, (), (), ForkDisabled>
{
  fn from(expectation: Expectation<'expect_literal, Kind>) -> Self {
    Self::new().expect(expectation)
  }
}
impl<'expect_literal, Kind> From<ReLexContext>
  for StatelessLexOptions<'expect_literal, Kind, (), (), ForkDisabled>
{
  fn from(re_lex: ReLexContext) -> Self {
    Self::new().re_lex(re_lex)
  }
}
impl<'expect_literal, Kind, Fork> From<LexOptions<'expect_literal, Kind, Fork>>
  for StatelessLexOptions<'expect_literal, Kind, (), (), Fork>
{
  fn from(base: LexOptions<'expect_literal, Kind, Fork>) -> Self {
    Self {
      start: 0,
      action_state: (),
      err_acc: (),
      base,
    }
  }
}

impl<'expect_literal, Kind, ActionStateRef, ErrAcc, Fork>
  StatelessLexOptions<'expect_literal, Kind, ActionStateRef, ErrAcc, Fork>
{
  /// The start index of the text to lex.
  pub fn start(mut self, start: usize) -> Self {
    self.start = start;
    self
  }

  /// Set the action state.
  pub fn action_state<NewActionStateRef>(
    self,
    action_state: NewActionStateRef,
  ) -> StatelessLexOptions<'expect_literal, Kind, NewActionStateRef, ErrAcc, Fork> {
    StatelessLexOptions {
      start: self.start,
      action_state,
      err_acc: self.err_acc,
      base: self.base,
    }
  }

  /// Set the error accumulator.
  pub fn err_acc<NewErrAcc>(
    self,
    err_acc: NewErrAcc,
  ) -> StatelessLexOptions<'expect_literal, Kind, ActionStateRef, NewErrAcc, Fork> {
    StatelessLexOptions {
      start: self.start,
      action_state: self.action_state,
      err_acc,
      base: self.base,
    }
  }
}

// re-export/override from `LexOptions`
// but with `StatelessLexOptions` as the return type
// instead of `LexOptions`
impl<'expect_literal, Kind, ActionStateRef, ErrAcc, Fork>
  StatelessLexOptions<'expect_literal, Kind, ActionStateRef, ErrAcc, Fork>
{
  /// See [`LexOptions::expect()`].
  pub fn expect(mut self, expectation: impl Into<Expectation<'expect_literal, Kind>>) -> Self
  where
    Kind: 'static,
  {
    self.base.expectation = expectation.into();
    self
  }
  /// See [`LexOptions::expect_with()`].
  pub fn expect_with(
    mut self,
    f: impl FnOnce(Expectation<'expect_literal, Kind>) -> Expectation<'expect_literal, Kind>,
  ) -> Self {
    self.base.expectation = f(Expectation::default());
    self
  }
  /// See [`LexOptions::fork()`].
  pub fn fork<ActionState>(
    self,
  ) -> StatelessLexOptions<'expect_literal, Kind, ActionStateRef, ErrAcc, ForkEnabled> {
    StatelessLexOptions {
      start: self.start,
      action_state: self.action_state,
      err_acc: self.err_acc,
      base: self.base.fork(),
    }
  }
  /// See [`LexOptions::re_lex()`].
  pub fn re_lex(mut self, re_lex: ReLexContext) -> Self {
    self.base.re_lex = re_lex;
    self
  }
}

pub struct StatelessTrimOptions<ActionStateRef, ErrAcc> {
  /// See [`StatelessTrimOptions::start()`].
  pub start: usize,
  /// This is usually `&mut ActionState`.
  pub action_state: ActionStateRef,
  pub err_acc: ErrAcc,
}

impl StatelessTrimOptions<(), ()> {
  pub fn new() -> Self {
    Self {
      start: 0,
      action_state: (), // use `()` as a placeholder, user should use `self.action_state` to set this
      err_acc: (),
    }
  }
}

impl<ActionStateRef, ErrAcc> StatelessTrimOptions<ActionStateRef, ErrAcc> {
  /// The start index of the text to trim.
  pub fn start(mut self, start: usize) -> Self {
    self.start = start;
    self
  }

  /// Set the action state.
  pub fn action_state<NewActionStateRef>(
    self,
    action_state: NewActionStateRef,
  ) -> StatelessTrimOptions<NewActionStateRef, ErrAcc> {
    StatelessTrimOptions {
      start: self.start,
      action_state,
      err_acc: self.err_acc,
    }
  }

  /// Set the error accumulator.
  pub fn err_acc<NewErrAcc>(
    self,
    err_acc: NewErrAcc,
  ) -> StatelessTrimOptions<ActionStateRef, NewErrAcc> {
    StatelessTrimOptions {
      start: self.start,
      action_state: self.action_state,
      err_acc,
    }
  }
}
