#[derive(Debug)]
pub struct LexOutput<TokenType, ErrAcc, ReLexableType> {
  pub token: Option<TokenType>,
  /// How many bytes are digested during the whole lexing loop in current lexing.
  /// This is NOT [`ActionOutput::digested`](crate::lexer::action::ActionOutput::digested)
  /// because there might be many actions which are accepted during multiple iterations
  /// of the lexing loop, this value is the sum of them.
  pub digested: usize,
  pub errors: ErrAcc,
  /// If [`Some`], the lex is re-lexable and you can use this value
  /// to continue a lex. This *might* be [`Some`] only if the [`LexOptions::fork`](super::options::LexOptions::fork)
  /// is enabled.
  pub re_lexable: ReLexableType, // TODO: example
}

pub struct TrimOutput<ErrAcc> {
  /// How many bytes are digested during the whole lexing loop in current lexing.
  /// This is NOT [`ActionOutput::digested`](crate::lexer::action::ActionOutput::digested)
  /// because there might be many actions which are accepted during multiple iterations
  /// of the lexing loop, this value is the sum of them.
  pub digested: usize,
  pub err_acc: ErrAcc,
}
