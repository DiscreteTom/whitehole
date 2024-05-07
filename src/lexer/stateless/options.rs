use crate::lexer::{
  expectation::Expectation,
  fork::{ForkDisabled, ForkEnabled},
  options::LexOptions,
  re_lex::ReLexContext,
};
use std::ops::{Deref, DerefMut};

pub struct StatelessLexOptions<'expect_text, Kind: 'static, ActionStateRef, Fork> {
  /// See [`StatelessLexOptions::start()`].
  pub start: usize,
  /// This is usually `&mut ActionState`.
  pub action_state: ActionStateRef,
  pub base: LexOptions<'expect_text, Kind, Fork>,
}

impl<'expect_text, Kind> Default for StatelessLexOptions<'expect_text, Kind, (), ForkDisabled> {
  fn default() -> Self {
    Self {
      start: 0,
      action_state: (), // use `()` as a placeholder, user should use `self.action_state` to set this
      base: LexOptions::default(),
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
impl<'expect_text, Kind> From<ReLexContext>
  for StatelessLexOptions<'expect_text, Kind, (), ForkDisabled>
{
  fn from(re_lex: ReLexContext) -> Self {
    Self::default().re_lex(re_lex)
  }
}
impl<'expect_text, Kind, Fork> From<LexOptions<'expect_text, Kind, Fork>>
  for StatelessLexOptions<'expect_text, Kind, (), Fork>
{
  fn from(base: LexOptions<'expect_text, Kind, Fork>) -> Self {
    Self {
      start: 0,
      action_state: (),
      base,
    }
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
    }
  }
}

// re-export/override from `LexOptions`
// but with `StatelessLexOptions` as the return type
// instead of `LexOptions`
impl<'expect_text, Kind, ActionStateRef, Fork>
  StatelessLexOptions<'expect_text, Kind, ActionStateRef, Fork>
{
  /// See [`LexOptions::expect()`].
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
  ) -> StatelessLexOptions<'expect_text, Kind, ActionStateRef, ForkEnabled> {
    StatelessLexOptions {
      start: self.start,
      action_state: self.action_state,
      base: self.base.fork(),
    }
  }
  /// See [`LexOptions::re_lex()`].
  pub fn re_lex(mut self, re_lex: ReLexContext) -> Self {
    self.re_lex = re_lex;
    self
  }
}
