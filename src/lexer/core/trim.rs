use super::{common::Validator, LexerCore};
use crate::lexer::{
  core::common::OutputHandler,
  token::{Token, TokenKind},
};
use std::rc::Rc;

pub struct TrimOutput<TokenType, TrimmedLexer> {
  pub digested: usize,
  pub errors: Vec<TokenType>,
  pub trimmed: TrimmedLexer,
}

pub struct LexerCoreTrimOutput<TokenType> {
  pub digested: usize,
  pub errors: Vec<TokenType>,
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static>
  LexerCore<Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  pub fn trim<'buffer, 'expect_text>(
    &mut self,
    buffer: &'buffer str,
    start: usize,
  ) -> LexerCoreTrimOutput<Rc<Token<'buffer, Kind, ErrorType>>>
  where
    'buffer: 'expect_text,
  {
    // use static to avoid allocation in each call
    static OUTPUT_HANDLER: OutputHandler = OutputHandler {
      update_lex_output: false,
      create_token: false,
    };

    let output = Self::execute_actions(
      &self.actions,
      move |_| Validator {
        skip_before_exec: Box::new(move |action| action.never_muted()),
        accept_after_exec: Box::new(move |_, _, _| true),
      },
      buffer,
      start,
      false,
      &mut self.state,
      &OUTPUT_HANDLER,
    );

    LexerCoreTrimOutput {
      digested: output.digested,
      errors: output.errors,
    }
  }
}
