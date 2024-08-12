use super::{expectation::Expectation, fork::ForkEnabled, re_lex::ReLexContext};
use crate::utils::AccumulatorSetter;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LexOptions<'expect_literal, Kind, ErrAcc, Fork> {
  /// See [`Self::expect`].
  pub expectation: Expectation<'expect_literal, Kind>,
  /// See [`Self::errors_to`].
  pub errors_to: ErrAcc,
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
      errors_to: (),
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
  /// # use whitehole::lexer::token::token_kind;
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
  /// # use whitehole::lexer::{options::LexOptions, LexerBuilder};
  /// // print errors to stdout (for debugging)
  /// # let options: LexOptions<(), Vec<()>, _> = LexOptions::new();
  /// options.errors().to_stdout();
  ///
  /// // if you want to collect errors in a vector,
  /// // you should provision the capacity
  /// // and re-use the vector to prevent unnecessary allocations
  /// let mut errors = Vec::with_capacity(16);
  /// # let mut lexer = LexerBuilder::new().build("");
  /// lexer.lex_with(|o| o.errors().to(&mut errors));
  /// errors.clear();
  /// lexer.lex_with(|o| o.errors().to(&mut errors));
  /// ```
  #[inline]
  pub fn errors<Acc>(
    self,
  ) -> AccumulatorSetter<impl FnOnce(Acc) -> LexOptions<'expect_literal, Kind, Acc, Fork>> {
    AccumulatorSetter::new(move |acc| LexOptions {
      expectation: self.expectation,
      errors_to: acc,
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
      errors_to: self.errors_to,
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
  /// See [`Self::errors_to`]
  pub errors_to: ErrAcc,
}

impl TrimOptions<()> {
  /// Create a new instance with no error accumulator.
  #[inline]
  pub const fn new() -> Self {
    Self { errors_to: () }
  }
}

impl<ErrAcc> TrimOptions<ErrAcc> {
  /// Set the error accumulator.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::options::TrimOptions;
  /// # let options: TrimOptions<Vec<()>> =
  /// TrimOptions::new().errors_to(vec![]);
  /// ```
  #[inline]
  pub fn errors_to<NewErrAcc>(self, acc: NewErrAcc) -> TrimOptions<NewErrAcc> {
    TrimOptions { errors_to: acc }
  }

  /// Collect the errors into a vector.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::options::TrimOptions;
  /// # let options: TrimOptions<Vec<()>> =
  /// TrimOptions::new().errors_to_vec();
  /// ```
  #[inline]
  pub fn errors_to_vec<E>(self) -> TrimOptions<Vec<E>> {
    self.errors_to(Vec::new()) // TODO: use a trait for `errors_to`
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
        errors_to: (),
        fork: (),
        re_lex: ReLexContext::new(),
      }
    );

    let options = options.expect("literal");
    assert_eq!(
      options,
      LexOptions {
        expectation: Expectation::new().literal("literal"),
        errors_to: (),
        fork: (),
        re_lex: ReLexContext::new(),
      }
    );

    let options: LexOptions<(), Vec<()>, _> = options.errors_to_vec();
    assert_eq!(
      options,
      LexOptions {
        expectation: Expectation::new().literal("literal"),
        errors_to: vec![],
        fork: (),
        re_lex: ReLexContext::new(),
      }
    );

    let options = options.fork();
    assert_eq!(
      options,
      LexOptions {
        expectation: Expectation::new().literal("literal"),
        errors_to: vec![],
        fork: ForkEnabled,
        re_lex: ReLexContext::new(),
      }
    );

    let options = options.re_lex(ReLexContext { start: 1, skip: 1 });
    assert_eq!(
      options,
      LexOptions {
        expectation: Expectation::new().literal("literal"),
        errors_to: vec![],
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
        errors_to: (),
        fork: (),
        re_lex: ReLexContext::new(),
      }
    );
    let options: LexOptions<(), _, _> = ReLexContext { start: 1, skip: 1 }.into();
    assert_eq!(
      options,
      LexOptions {
        expectation: Expectation::new(),
        errors_to: (),
        fork: (),
        re_lex: ReLexContext { start: 1, skip: 1 },
      }
    );
  }

  #[test]
  fn trim_options() {
    let options = TrimOptions::new();
    assert_eq!(options, TrimOptions { errors_to: () });

    let options: TrimOptions<Vec<()>> = options.errors_to_vec();
    assert_eq!(options, TrimOptions { errors_to: vec![] });
  }
}
