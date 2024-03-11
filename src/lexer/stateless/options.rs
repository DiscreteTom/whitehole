use crate::lexer::{
  expectation::Expectation,
  options::{LexOptions, ReLexContext},
};
use std::ops::{Deref, DerefMut};

pub struct StatelessLexOptions<'expect_text, Kind> {
  /// The start index of the text to lex.
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
    Self {
      start: 0,
      base: expectation.into(),
    }
  }
}

impl<'expect_text, Kind> From<ReLexContext> for StatelessLexOptions<'expect_text, Kind> {
  fn from(re_lex: ReLexContext) -> Self {
    Self {
      start: 0,
      base: re_lex.into(),
    }
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

impl<'expect_text, Kind> DerefMut for StatelessLexOptions<'expect_text, Kind> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.base
  }
}

impl<'expect_text, Kind> Deref for StatelessLexOptions<'expect_text, Kind> {
  type Target = LexOptions<'expect_text, Kind>;

  fn deref(&self) -> &Self::Target {
    &self.base
  }
}

impl<'expect_text, Kind> StatelessLexOptions<'expect_text, Kind> {
  /// The start index of the text to lex.
  pub fn start(mut self, start: usize) -> Self {
    self.start = start;
    self
  }
  pub fn expect(mut self, expectation: impl Into<Expectation<'expect_text, Kind>>) -> Self {
    self.expectation = expectation.into();
    self
  }
  /// Provide this if the lex is a re-lex.
  pub fn re_lex(mut self, re_lex: ReLexContext) -> Self {
    self.re_lex = Some(re_lex);
    self
  }
}
