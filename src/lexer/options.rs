use super::{expectation::Expectation, fork::ForkEnabled, re_lex::ReLexContext};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LexOptions<'literal, Kind, Fork> {
  /// See [`Self::expect`].
  pub expectation: Expectation<'literal, Kind>,
  /// See [`Self::fork`].
  pub fork: Fork,
  /// See [`Self::re_lex`].
  pub re_lex: ReLexContext,
}

impl<'literal, Kind> LexOptions<'literal, Kind, ()> {
  /// Create a new instance with no expectation, no error accumulator, no re-lex context and fork disabled.
  #[inline]
  pub const fn new() -> Self {
    Self {
      expectation: Expectation::new(),
      fork: (),
      re_lex: ReLexContext::new(),
    }
  }
}

impl<'literal, Kind> Default for LexOptions<'literal, Kind, ()> {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

impl<'literal, Kind> From<Expectation<'literal, Kind>> for LexOptions<'literal, Kind, ()> {
  #[inline]
  fn from(expectation: Expectation<'literal, Kind>) -> Self {
    Self::new().expect(expectation)
  }
}
impl<'literal, Kind> From<ReLexContext> for LexOptions<'literal, Kind, ()> {
  #[inline]
  fn from(re_lex: ReLexContext) -> Self {
    Self::new().re_lex(re_lex)
  }
}

impl<'literal, Kind, Fork> LexOptions<'literal, Kind, Fork> {
  /// Set the expectation to speed up the lexing.
  /// See [`Expectation`] for more details.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::options::LexOptions;
  /// # use whitehole::lexer::expectation::Expectation;
  /// # use whitehole::kind::{whitehole_kind, SubKind};
  /// #[whitehole_kind]
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
  pub fn expect(mut self, expectation: impl Into<Expectation<'literal, Kind>>) -> Self {
    self.expectation = expectation.into();
    self
  }

  /// Set the expectation to speed up the lexing.
  /// See [`Expectation`] for more details.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::options::LexOptions;
  /// # use whitehole::kind::{whitehole_kind, SubKind};
  /// #[whitehole_kind]
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
    f: impl FnOnce(Expectation<'literal, Kind>) -> Expectation<'literal, Kind>,
  ) -> Self {
    self.expectation = f(Expectation::default());
    self
  }

  /// If set, and the lex is re-lexable (the accepted action is not the last candidate action),
  /// you can use [`LexOutput::fork`](crate::lexer::output::LexOutput::fork)
  /// to re-try the lex with different actions.
  ///
  /// See [`ReLexContext`] for more details.
  #[inline]
  pub fn fork(self) -> LexOptions<'literal, Kind, ForkEnabled> {
    LexOptions {
      expectation: self.expectation,
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

// currently there is no fields in TrimOptions.
// we define this for future compatibility.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct TrimOptions;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn lex_options() {
    let options: LexOptions<(), _> = LexOptions::new();
    assert_eq!(
      options,
      LexOptions {
        expectation: Expectation::new(),

        fork: (),
        re_lex: ReLexContext::new(),
      }
    );

    let options = options.expect("literal");
    assert_eq!(
      options,
      LexOptions {
        expectation: Expectation::new().literal("literal"),

        fork: (),
        re_lex: ReLexContext::new(),
      }
    );

    let options = options.fork();
    assert_eq!(
      options,
      LexOptions {
        expectation: Expectation::new().literal("literal"),
        fork: ForkEnabled,
        re_lex: ReLexContext::new(),
      }
    );

    let options = options.re_lex(ReLexContext { start: 1, skip: 1 });
    assert_eq!(
      options,
      LexOptions {
        expectation: Expectation::new().literal("literal"),
        fork: ForkEnabled,
        re_lex: ReLexContext { start: 1, skip: 1 },
      }
    );

    // from
    let options: LexOptions<(), _> = Expectation::new().literal("literal").into();
    assert_eq!(
      options,
      LexOptions {
        expectation: Expectation::new().literal("literal"),
        fork: (),
        re_lex: ReLexContext::new(),
      }
    );
    let options: LexOptions<(), _> = ReLexContext { start: 1, skip: 1 }.into();
    assert_eq!(
      options,
      LexOptions {
        expectation: Expectation::new(),
        fork: (),
        re_lex: ReLexContext { start: 1, skip: 1 },
      }
    );
  }
}
