use crate::lexer::output::{LexOutput, TrimOutput};

/// Use this trait to abstract the output of a stateless lexer.
pub(super) trait StatelessOutput<TokenType, ErrAcc, ReLexableType> {
  /// Create a new instance of the output with the given error accumulator.
  fn new(errors: ErrAcc) -> Self;
  /// How many bytes are digested during the whole lexing loop in current lexing.
  fn digested(&self) -> usize;
  /// Digest the next `n` chars.
  fn digest(&mut self, n: usize);
  /// Get a mutable reference to the error accumulator.
  fn errors(&mut self) -> &mut ErrAcc;
  /// This is called if an un-muted action is accepted.
  fn token(&mut self, token: TokenType, re_lexable: ReLexableType);
}

impl<TokenType, ErrAcc, ReLexableType: Default> StatelessOutput<TokenType, ErrAcc, ReLexableType>
  for LexOutput<TokenType, ErrAcc, ReLexableType>
{
  #[inline]
  fn new(errors: ErrAcc) -> Self {
    Self {
      token: None,
      digested: 0,
      errors,
      re_lexable: ReLexableType::default(),
    }
  }
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
  fn token(&mut self, token: TokenType, re_lexable: ReLexableType) {
    self.token = Some(token);
    self.re_lexable = re_lexable;
  }
}

impl<TokenType, ReLexableType, ErrAcc> StatelessOutput<TokenType, ErrAcc, ReLexableType>
  for TrimOutput<ErrAcc>
{
  #[inline]
  fn new(errors: ErrAcc) -> Self {
    Self {
      digested: 0,
      errors,
    }
  }
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
  fn token(&mut self, _token: TokenType, _re_lexable: ReLexableType) {
    // this will never be called
    // because when trim, all actions are muted, no token will be emitted
    unreachable!()
  }
}
