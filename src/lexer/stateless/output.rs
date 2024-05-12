use crate::lexer::output::{LexOutput, TrimOutput};

pub(super) trait StatelessOutput<TokenType, ReLexableType>: Default {
  fn digested(&self) -> usize;
  fn digest(&mut self, n: usize);
  fn append_error_token(&mut self, token: TokenType);
  /// This is called if an un-muted action is accepted.
  fn emit(&mut self, token: TokenType, re_lex: ReLexableType);
}

impl<TokenType, ReLexableType: Default> StatelessOutput<TokenType, ReLexableType>
  for LexOutput<TokenType, ReLexableType>
{
  fn digested(&self) -> usize {
    self.digested
  }

  fn digest(&mut self, n: usize) {
    self.digested += n;
  }

  fn append_error_token(&mut self, token: TokenType) {
    self.errors.push(token);
  }

  fn emit(&mut self, token: TokenType, re_lex: ReLexableType) {
    self.token = Some(token);
    self.re_lex = re_lex;
  }
}

impl<TokenType, ReLexableType> StatelessOutput<TokenType, ReLexableType> for TrimOutput<TokenType> {
  fn digested(&self) -> usize {
    self.digested
  }

  fn digest(&mut self, n: usize) {
    self.digested += n;
  }

  fn append_error_token(&mut self, token: TokenType) {
    self.errors.push(token);
  }

  fn emit(&mut self, _token: TokenType, _re_lex: ReLexableType) {
    // this will never be called
    panic!("TrimOutput::done should never be called");
  }
}
