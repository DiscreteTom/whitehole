use crate::lexer::{
  expectation::Expectation,
  options::{ForkDisabled, ForkEnabled, LexOptions, LexOptionsFork, ReLexContext},
};
use std::ops::{Deref, DerefMut};

pub struct StatelessLexOptions<
  'expect_text,
  Kind: 'static,
  ActionState,
  ActionStateRef,
  Fork: LexOptionsFork<ActionState>,
> {
  /// See [`StatelessLexOptions::start()`].
  pub start: usize,
  /// This is usually `&mut ActionState`.
  pub action_state: ActionStateRef,
  pub base: LexOptions<'expect_text, Kind, ActionState, Fork>,
}

impl<'expect_text, Kind, ActionState> Default
  for StatelessLexOptions<'expect_text, Kind, ActionState, (), ForkDisabled>
{
  fn default() -> Self {
    Self {
      start: 0,
      action_state: (), // use `()` as a placeholder, user should use `self.action_state` to set this
      base: LexOptions::default(),
    }
  }
}

impl<'expect_text, Kind, ActionState> From<Expectation<'expect_text, Kind>>
  for StatelessLexOptions<'expect_text, Kind, ActionState, (), ForkDisabled>
{
  fn from(expectation: Expectation<'expect_text, Kind>) -> Self {
    Self::default().expect(expectation)
  }
}

impl<'expect_text, Kind, ActionState> From<ReLexContext>
  for StatelessLexOptions<'expect_text, Kind, ActionState, (), ForkDisabled>
{
  fn from(re_lex: ReLexContext) -> Self {
    Self::default().re_lex(re_lex)
  }
}

impl<'expect_text, Kind, ActionState, Fork: LexOptionsFork<ActionState>>
  From<LexOptions<'expect_text, Kind, ActionState, Fork>>
  for StatelessLexOptions<'expect_text, Kind, ActionState, (), Fork>
{
  fn from(options: LexOptions<'expect_text, Kind, ActionState, Fork>) -> Self {
    Self {
      start: 0,
      action_state: (),
      base: options,
    }
  }
}

impl<'expect_text, Kind: 'static, ActionState, ActionStateRef, Fork: LexOptionsFork<ActionState>>
  Deref for StatelessLexOptions<'expect_text, Kind, ActionState, ActionStateRef, Fork>
{
  type Target = LexOptions<'expect_text, Kind, ActionState, Fork>;

  fn deref(&self) -> &Self::Target {
    &self.base
  }
}

impl<'expect_text, Kind: 'static, ActionState, ActionStateRef, Fork: LexOptionsFork<ActionState>>
  DerefMut for StatelessLexOptions<'expect_text, Kind, ActionState, ActionStateRef, Fork>
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.base
  }
}

impl<'expect_text, Kind, ActionState, ActionStateRef, Fork: LexOptionsFork<ActionState>>
  StatelessLexOptions<'expect_text, Kind, ActionState, ActionStateRef, Fork>
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
  ) -> StatelessLexOptions<'expect_text, Kind, ActionState, NewActionStateRef, Fork> {
    StatelessLexOptions {
      start: self.start,
      action_state,
      base: self.base,
    }
  }

  // action state is a mutable ref so the default is meaningless
  /// Set the action state to default.
  // pub fn default_action_state<NewActionState>(
  //   self,
  // ) -> StatelessLexOptions<'expect_text, Kind, NewActionState, Fork>
  // where
  //   NewActionState: Default,
  // {
  //   self.action_state(NewActionState::default())
  // }

  // re-export from `LexOptions` but with `self` return type
  pub fn expect(mut self, expectation: impl Into<Expectation<'expect_text, Kind>>) -> Self
  where
    Kind: 'static,
  {
    self.expectation = expectation.into();
    self
  }
  /// See [`LexOptions::fork()`].
  pub fn fork(
    self,
  ) -> StatelessLexOptions<'expect_text, Kind, ActionState, ActionStateRef, ForkEnabled<ActionState>>
  where
    ActionState: Clone,
  {
    StatelessLexOptions {
      start: self.start,
      action_state: self.action_state,
      base: self.base.fork(),
    }
  }
  /// See [`LexOptions::re_lex()`].
  pub fn re_lex(mut self, re_lex: ReLexContext) -> Self {
    self.re_lex = Some(re_lex);
    self
  }
}
