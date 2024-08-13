use super::{expectation::Expectation, fork::ForkEnabled, re_lex::ReLexContext};
use crate::utils::AccumulatorSetter;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LexOptions<'expect_literal, Kind, ErrAcc, Fork> {
  /// See [`Self::expect`].
  pub expectation: Expectation<'expect_literal, Kind>,
  /// See [`Self::errors`].
  pub errors: ErrAcc,
  /// See [`Self::fork`].
  pub fork: Fork,
  /// See [`Self::re_lex`].
  pub re_lex: ReLexContext,
}

impl<'expect_literal, Kind> LexOptions<'expect_literal, Kind, (), ()> {
  /// Create a new instance with no expectation, no error accumulator, no re-lex context and fork disabled.
  #[inline]
  pub const fn new() -> Self {
    Self {
      expectation: Expectation::new(),
      errors: (),
      fork: (),
      re_lex: ReLexContext::new(),
    }
  }
}

impl<'expect_literal, Kind> From<Expectation<'expect_literal, Kind>>
  for LexOptions<'expect_literal, Kind, (), ()>
{
  #[inline]
  fn from(expectation: Expectation<'expect_literal, Kind>) -> Self {
    Self::new().expect(expectation)
  }
}
impl<'expect_literal, Kind> From<ReLexContext> for LexOptions<'expect_literal, Kind, (), ()> {
  #[inline]
  fn from(re_lex: ReLexContext) -> Self {
    Self::new().re_lex(re_lex)
  }
}

impl<'expect_literal, Kind, ErrAcc, Fork> LexOptions<'expect_literal, Kind, ErrAcc, Fork> {
  /// Set the expectation to speed up the lexing.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::options::LexOptions;
  /// # use whitehole::lexer::expectation::Expectation;
  /// # use whitehole::lexer::token::{token_kind, SubTokenKind};
  /// #[token_kind]
  /// enum MyKind { A }
  ///
  /// # fn main() {
  /// # let options = LexOptions::new();
  /// // with a kind
  /// # let options =
  /// options.expect(A::kind_id());
  /// # let options =
  /// options.expect(A);
  ///
  /// // with a literal
  /// # let options =
  /// options.expect("literal");
  ///
  /// // with both
  /// # let options =
  /// options.expect(Expectation::new().kind(A).literal("literal"));
  /// # }
  /// ```
  #[inline]
  pub fn expect(mut self, expectation: impl Into<Expectation<'expect_literal, Kind>>) -> Self {
    self.expectation = expectation.into();
    self
  }

  /// Set the expectation to speed up the lexing.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::options::LexOptions;
  /// # use whitehole::lexer::token::{token_kind, SubTokenKind};
  /// #[token_kind]
  /// enum MyKind { A }
  ///
  /// # fn main() {
  /// # let options = LexOptions::new();
  ///
  /// // with a kind
  /// # let options =
  /// options.expect_with(|e| e.kind(A::kind_id()));
  /// # let options =
  /// options.expect_with(|e| e.kind(A));
  ///
  /// // with a literal
  /// # let options =
  /// options.expect_with(|e| e.literal("literal"));
  ///
  /// // expect both kind and literal
  /// options.expect_with(|e| e.kind(A).literal("literal"));
  /// # }
  /// ```
  #[inline]
  pub fn expect_with(
    mut self,
    f: impl FnOnce(Expectation<'expect_literal, Kind>) -> Expectation<'expect_literal, Kind>,
  ) -> Self {
    self.expectation = f(Expectation::default());
    self
  }

  /// Set the error accumulator.
  /// By default error accumulator is `()` and all errors will be discarded.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{options::LexOptions, LexerBuilder, Lexer, token::MockTokenKind};
  /// // print errors to stdout (for debugging)
  /// # let options: LexOptions<(), _, _> = LexOptions::new();
  /// options.errors().to_stdout();
  ///
  /// // if you want to collect errors in a vector,
  /// // you should provision the capacity
  /// // and re-use the vector to prevent unnecessary allocations
  /// let mut errors = Vec::with_capacity(16);
  /// # let mut lexer: Lexer<MockTokenKind<()>> = LexerBuilder::new().build("");
  /// lexer.lex_with(|o| o.errors().to(&mut errors));
  /// errors.clear();
  /// lexer.lex_with(|o| o.errors().to(&mut errors));
  /// ```
  #[inline]
  pub fn errors<NewErrAcc>(
    self,
  ) -> AccumulatorSetter<impl FnOnce(NewErrAcc) -> LexOptions<'expect_literal, Kind, NewErrAcc, Fork>>
  {
    AccumulatorSetter::new(move |acc| LexOptions {
      expectation: self.expectation,
      errors: acc,
      fork: self.fork,
      re_lex: self.re_lex,
    })
  }

  /// If set, and the lex is re-lexable (the accepted action is not the last candidate action),
  /// you can use [`LexOutput::fork`](crate::lexer::output::LexOutput::fork)
  /// to re-try the lex with different actions.
  ///
  /// See [`ReLexContext`] for more details.
  #[inline]
  pub fn fork(self) -> LexOptions<'expect_literal, Kind, ErrAcc, ForkEnabled> {
    LexOptions {
      expectation: self.expectation,
      errors: self.errors,
      fork: ForkEnabled,
      re_lex: self.re_lex,
    }
  }

  /// Provide the [`ReLexContext`] to retry a lexing.
  ///
  /// See [`ReLexContext`] for more details.
  #[inline]
  pub fn re_lex(mut self, re_lex: ReLexContext) -> Self {
    self.re_lex = re_lex;
    self
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrimOptions<ErrAcc> {
  /// See [`Self::errors`]
  pub errors: ErrAcc,
}

impl TrimOptions<()> {
  /// Create a new instance with no error accumulator.
  #[inline]
  pub const fn new() -> Self {
    Self { errors: () }
  }
}

impl<ErrAcc> TrimOptions<ErrAcc> {
  /// Set the error accumulator.
  /// By default error accumulator is `()` and all errors will be discarded.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{options::TrimOptions, LexerBuilder, Lexer, token::MockTokenKind};
  /// // print errors to stdout (for debugging)
  /// # let options = TrimOptions::new();
  /// options.errors().to_stdout();
  ///
  /// // if you want to collect errors in a vector,
  /// // you should provision the capacity
  /// // and re-use the vector to prevent unnecessary allocations
  /// let mut errors = Vec::with_capacity(16);
  /// # let mut lexer: Lexer<MockTokenKind<()>> = LexerBuilder::new().build("");
  /// lexer.trim_with(|o| o.errors().to(&mut errors));
  /// errors.clear();
  /// lexer.trim_with(|o| o.errors().to(&mut errors));
  /// ```
  #[inline]
  pub fn errors<NewErrAcc>(
    self,
  ) -> AccumulatorSetter<impl FnOnce(NewErrAcc) -> TrimOptions<NewErrAcc>> {
    AccumulatorSetter::new(move |acc| TrimOptions { errors: acc })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn lex_options() {
    let options: LexOptions<(), _, _> = LexOptions::new();
    assert_eq!(
      options,
      LexOptions {
        expectation: Expectation::new(),
        errors: (),
        fork: (),
        re_lex: ReLexContext::new(),
      }
    );

    let options = options.expect("literal");
    assert_eq!(
      options,
      LexOptions {
        expectation: Expectation::new().literal("literal"),
        errors: (),
        fork: (),
        re_lex: ReLexContext::new(),
      }
    );

    let mut errors = vec![];
    let options: LexOptions<(), &mut Vec<()>, _> = options.errors().to(&mut errors);
    assert_eq!(
      options,
      LexOptions {
        expectation: Expectation::new().literal("literal"),
        errors: &mut vec![],
        fork: (),
        re_lex: ReLexContext::new(),
      }
    );

    let options = options.fork();
    assert_eq!(
      options,
      LexOptions {
        expectation: Expectation::new().literal("literal"),
        errors: &mut vec![],
        fork: ForkEnabled,
        re_lex: ReLexContext::new(),
      }
    );

    let options = options.re_lex(ReLexContext { start: 1, skip: 1 });
    assert_eq!(
      options,
      LexOptions {
        expectation: Expectation::new().literal("literal"),
        errors: &mut vec![],
        fork: ForkEnabled,
        re_lex: ReLexContext { start: 1, skip: 1 },
      }
    );

    // from
    let options: LexOptions<(), _, _> = Expectation::new().literal("literal").into();
    assert_eq!(
      options,
      LexOptions {
        expectation: Expectation::new().literal("literal"),
        errors: (),
        fork: (),
        re_lex: ReLexContext::new(),
      }
    );
    let options: LexOptions<(), _, _> = ReLexContext { start: 1, skip: 1 }.into();
    assert_eq!(
      options,
      LexOptions {
        expectation: Expectation::new(),
        errors: (),
        fork: (),
        re_lex: ReLexContext { start: 1, skip: 1 },
      }
    );
  }

  #[test]
  fn trim_options() {
    let options = TrimOptions::new();
    assert_eq!(options, TrimOptions { errors: () });

    let mut errors = vec![];
    let options: TrimOptions<&mut Vec<()>> = options.errors().to(&mut errors);
    assert_eq!(
      options,
      TrimOptions {
        errors: &mut vec![]
      }
    );
  }
}
