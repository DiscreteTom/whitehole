use crate::lexer::{
  expectation::Expectation,
  options::{LexOptions, ReLexContext},
};
use std::ops::{Deref, DerefMut};

pub struct StatelessLexOptions<'expect_text, Kind: 'static> {
  /// See [`StatelessLexOptions::start()`].
  pub start: usize,
  pub base: LexOptions<'expect_text, Kind>,
}

impl<'expect_text, Kind> Default for StatelessLexOptions<'expect_text, Kind> {
  fn default() -> Self {
    Self {
      start: 0,
      base: LexOptions::default(),
    }
  }
}

impl<'expect_text, Kind> From<Expectation<'expect_text, Kind>>
  for StatelessLexOptions<'expect_text, Kind>
{
  fn from(expectation: Expectation<'expect_text, Kind>) -> Self {
    Self::default().expect(expectation)
  }
}

impl<'expect_text, Kind> From<ReLexContext> for StatelessLexOptions<'expect_text, Kind> {
  fn from(re_lex: ReLexContext) -> Self {
    Self::default().re_lex(re_lex)
  }
}

impl<'expect_text, Kind> From<LexOptions<'expect_text, Kind>>
  for StatelessLexOptions<'expect_text, Kind>
{
  fn from(options: LexOptions<'expect_text, Kind>) -> Self {
    Self {
      start: 0,
      base: options,
    }
  }
}

impl<'expect_text, Kind: 'static> Deref for StatelessLexOptions<'expect_text, Kind> {
  type Target = LexOptions<'expect_text, Kind>;

  fn deref(&self) -> &Self::Target {
    &self.base
  }
}

impl<'expect_text, Kind: 'static> DerefMut for StatelessLexOptions<'expect_text, Kind> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.base
  }
}

impl<'expect_text, Kind> StatelessLexOptions<'expect_text, Kind> {
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
  pub fn fork(mut self) -> Self {
    self.fork = true;
    self
  }
  /// See [`LexOptions::re_lex()`].
  pub fn re_lex(mut self, re_lex: ReLexContext) -> Self {
    self.re_lex = Some(re_lex);
    self
  }
}
