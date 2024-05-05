use crate::lexer::{
  expectation::Expectation,
  options::{ForkDisabled, ForkEnabled, LexOptions, LexOptionsFork, ReLexContext},
};
use std::ops::{Deref, DerefMut};

pub struct StatelessLexOptions<'expect_text, Kind: 'static, Fork: LexOptionsFork> {
  /// See [`StatelessLexOptions::start()`].
  pub start: usize,
  pub base: LexOptions<'expect_text, Kind, Fork>,
}

impl<'expect_text, Kind> Default for StatelessLexOptions<'expect_text, Kind, ForkDisabled> {
  fn default() -> Self {
    Self {
      start: 0,
      base: LexOptions::default(),
    }
  }
}

impl<'expect_text, Kind> From<Expectation<'expect_text, Kind>>
  for StatelessLexOptions<'expect_text, Kind, ForkDisabled>
{
  fn from(expectation: Expectation<'expect_text, Kind>) -> Self {
    Self::default().expect(expectation)
  }
}

impl<'expect_text, Kind> From<ReLexContext>
  for StatelessLexOptions<'expect_text, Kind, ForkDisabled>
{
  fn from(re_lex: ReLexContext) -> Self {
    Self::default().re_lex(re_lex)
  }
}

impl<'expect_text, Kind, Fork: LexOptionsFork> From<LexOptions<'expect_text, Kind, Fork>>
  for StatelessLexOptions<'expect_text, Kind, Fork>
{
  fn from(options: LexOptions<'expect_text, Kind, Fork>) -> Self {
    Self {
      start: 0,
      base: options,
    }
  }
}

impl<'expect_text, Kind: 'static, Fork: LexOptionsFork> Deref
  for StatelessLexOptions<'expect_text, Kind, Fork>
{
  type Target = LexOptions<'expect_text, Kind, Fork>;

  fn deref(&self) -> &Self::Target {
    &self.base
  }
}

impl<'expect_text, Kind: 'static, Fork: LexOptionsFork> DerefMut
  for StatelessLexOptions<'expect_text, Kind, Fork>
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.base
  }
}

impl<'expect_text, Kind, Fork: LexOptionsFork> StatelessLexOptions<'expect_text, Kind, Fork> {
  /// The start index of the text to lex.
  pub fn start(mut self, start: usize) -> Self {
    self.start = start;
    self
  }

  // re-export from `LexOptions` but with `self` return type
  pub fn expect(mut self, expectation: impl Into<Expectation<'expect_text, Kind>>) -> Self
  where
    Kind: 'static,
  {
    self.expectation = expectation.into();
    self
  }
  /// See [`LexOptions::fork()`].
  pub fn fork(self) -> StatelessLexOptions<'expect_text, Kind, ForkEnabled> {
    StatelessLexOptions {
      start: self.start,
      base: LexOptions {
        expectation: self.base.expectation,
        fork: ForkEnabled,
        re_lex: self.base.re_lex,
      },
    }
  }
  /// See [`LexOptions::re_lex()`].
  pub fn re_lex(mut self, re_lex: ReLexContext) -> Self {
    self.re_lex = Some(re_lex);
    self
  }
}
