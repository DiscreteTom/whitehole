use crate::lexer::output::{LexOutput, TrimOutput};

pub(super) trait StatelessOutput<TokenType, ReLexableType>: Default {
  fn digested(&self) -> usize;
  fn digest(&mut self, n: usize);
  fn append_error_token(&mut self, token: TokenType);
  /// This is called if an un-muted action is accepted.
  fn emit(&mut self, token: TokenType, re_lexable: ReLexableType);
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

  fn emit(&mut self, token: TokenType, re_lexable: ReLexableType) {
    self.token = Some(token);
    self.re_lexable = re_lexable;
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

  fn emit(&mut self, _token: TokenType, _re_lexable: ReLexableType) {
    // this will never be called
    // because when trim, all actions are muted
    panic!("TrimOutput::done should never be called");
  }
}
