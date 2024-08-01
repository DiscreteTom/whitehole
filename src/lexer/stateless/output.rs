use crate::lexer::output::{LexOutput, TrimOutput};

/// Use this trait to abstract the output of a stateless lexer.
pub(super) trait StatelessOutputFactory<TokenType, ErrAcc, ReLexableType> {
  type Target;
  /// How many bytes are digested during the whole lexing loop in current lexing.
  fn digested(&self) -> usize;
  /// Digest the next `n` chars.
  fn digest(&mut self, n: usize);
  /// Get a mutable reference to the error accumulator.
  fn errors(&mut self) -> &mut ErrAcc;
  /// This is called if no token is emitted.
  fn emit(self) -> Self::Target;
  /// This is called if an un-muted action is accepted.
  fn emit_with_token(self, token: TokenType, re_lexable: ReLexableType) -> Self::Target;
}

/// A helper struct to build [`LexOutput`].
pub(super) struct LexOutputBuilder<ErrAcc> {
  /// See [`LexOutput::digested`].
  digested: usize,
  /// See [`LexOutput::errors`].
  errors: ErrAcc,
}

impl<ErrAcc> LexOutputBuilder<ErrAcc> {
  #[inline]
  pub fn new(errors: ErrAcc) -> Self {
    Self {
      digested: 0,
      errors,
    }
  }
}

impl<TokenType, ErrAcc, ReLexableType: Default>
  StatelessOutputFactory<TokenType, ErrAcc, ReLexableType> for LexOutputBuilder<ErrAcc>
{
  type Target = LexOutput<TokenType, ErrAcc, ReLexableType>;

  #[inline]
  fn digested(&self) -> usize {
    self.digested
  }

  #[inline]
  fn digest(&mut self, n: usize) {
    self.digested += n;
  }

  #[inline]
  fn errors(&mut self) -> &mut ErrAcc {
    &mut self.errors
  }

  #[inline]
  fn emit(self) -> Self::Target {
    Self::Target {
      digested: self.digested,
      token: None,
      re_lexable: ReLexableType::default(),
      errors: self.errors,
    }
  }

  #[inline]
  fn emit_with_token(self, token: TokenType, re_lexable: ReLexableType) -> Self::Target {
    Self::Target {
      digested: self.digested,
      token: Some(token),
      re_lexable,
      errors: self.errors,
    }
  }
}

impl<TokenType, ReLexableType, ErrAcc> StatelessOutputFactory<TokenType, ErrAcc, ReLexableType>
  for TrimOutput<ErrAcc>
{
  type Target = Self;

  #[inline]
  fn digested(&self) -> usize {
    self.digested
  }

  #[inline]
  fn digest(&mut self, n: usize) {
    self.digested += n;
  }

  #[inline]
  fn errors(&mut self) -> &mut ErrAcc {
    &mut self.errors
  }

  #[inline]
  fn emit(self) -> Self::Target {
    self
  }

  #[inline]
  fn emit_with_token(self, _token: TokenType, _re_lexable: ReLexableType) -> Self::Target {
    // this will never be called
    // because when trim, all actions are muted, no token will be emitted
    unreachable!()
  }
}
