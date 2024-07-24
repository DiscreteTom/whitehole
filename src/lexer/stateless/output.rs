use crate::lexer::output::{LexOutput, TrimOutput};

pub(super) trait StatelessOutput<TokenType, ErrAcc, ReLexableType> {
  fn with_err_acc(err_acc: ErrAcc) -> Self;
  fn digested(&self) -> usize;
  fn digest(&mut self, n: usize);
  fn err_acc_mut(&mut self) -> &mut ErrAcc;
  /// This is called if an un-muted action is accepted.
  fn emit(&mut self, token: TokenType, re_lexable: ReLexableType);
}

impl<TokenType, ErrAcc, ReLexableType: Default> StatelessOutput<TokenType, ErrAcc, ReLexableType>
  for LexOutput<TokenType, ErrAcc, ReLexableType>
{
  fn with_err_acc(err_acc: ErrAcc) -> Self {
    Self {
      token: None,
      digested: 0,
      errors: err_acc,
      re_lexable: ReLexableType::default(),
    }
  }
  fn digested(&self) -> usize {
    self.digested
  }

  fn digest(&mut self, n: usize) {
    self.digested += n;
  }

  fn err_acc_mut(&mut self) -> &mut ErrAcc {
    &mut self.errors
  }

  fn emit(&mut self, token: TokenType, re_lexable: ReLexableType) {
    self.token = Some(token);
    self.re_lexable = re_lexable;
  }
}

impl<TokenType, ReLexableType, ErrAcc> StatelessOutput<TokenType, ErrAcc, ReLexableType>
  for TrimOutput<ErrAcc>
{
  fn with_err_acc(err_acc: ErrAcc) -> Self {
    Self {
      digested: 0,
      err_acc,
    }
  }
  fn digested(&self) -> usize {
    self.digested
  }

  fn digest(&mut self, n: usize) {
    self.digested += n;
  }

  fn err_acc_mut(&mut self) -> &mut ErrAcc {
    &mut self.err_acc
  }

  fn emit(&mut self, _token: TokenType, _re_lexable: ReLexableType) {
    // this will never be called
    // because when trim, all actions are muted
    unreachable!("TrimOutput::done should never be called")
  }
}
