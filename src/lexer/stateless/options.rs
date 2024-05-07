use crate::lexer::{
  expectation::Expectation,
  fork::{ForkDisabled, ForkEnabled},
  options::LexOptions,
  re_lex::ReLexContext,
};
use std::ops::{Deref, DerefMut};

pub struct StatelessLexOptions<'expect_text, Kind: 'static, ActionStateRef, Fork> {
  /// See [`StatelessLexOptions::start()`].
  pub start: usize, // TODO: replace with LexerState?
  /// This is usually `&mut ActionState`.
  pub action_state: ActionStateRef,
  pub base: LexOptions<'expect_text, Kind, Fork>,
  pub re_lex: ReLexContext,
}

impl<'expect_text, Kind> Default for StatelessLexOptions<'expect_text, Kind, (), ForkDisabled> {
  fn default() -> Self {
    Self {
      start: 0,
      action_state: (), // use `()` as a placeholder, user should use `self.action_state` to set this
      base: LexOptions::default(),
      re_lex: ReLexContext::default(),
    }
  }
}

impl<'expect_text, Kind> From<Expectation<'expect_text, Kind>>
  for StatelessLexOptions<'expect_text, Kind, (), ForkDisabled>
{
  fn from(expectation: Expectation<'expect_text, Kind>) -> Self {
    Self::default().expect(expectation)
  }
}

impl<'expect_text, Kind: 'static, ActionStateRef, Fork> Deref
  for StatelessLexOptions<'expect_text, Kind, ActionStateRef, Fork>
{
  type Target = LexOptions<'expect_text, Kind, Fork>;

  fn deref(&self) -> &Self::Target {
    &self.base
  }
}

impl<'expect_text, Kind: 'static, ActionStateRef, Fork> DerefMut
  for StatelessLexOptions<'expect_text, Kind, ActionStateRef, Fork>
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.base
  }
}

impl<'expect_text, Kind, ActionStateRef, Fork>
  StatelessLexOptions<'expect_text, Kind, ActionStateRef, Fork>
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
  ) -> StatelessLexOptions<'expect_text, Kind, NewActionStateRef, Fork> {
    StatelessLexOptions {
      start: self.start,
      action_state,
      base: self.base,
      re_lex: self.re_lex,
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
  pub fn fork<ActionState>(
    self,
  ) -> StatelessLexOptions<'expect_text, Kind, ActionStateRef, ForkEnabled<ActionState>>
  where
    ActionState: Clone,
  {
    StatelessLexOptions {
      start: self.start,
      action_state: self.action_state,
      base: self.base.fork(),
      re_lex: self.re_lex,
    }
  }
}
