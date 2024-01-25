use super::{common::Validator, LexerCore};
use crate::lexer::{
  core::common::OutputHandler,
  token::{Token, TokenKind},
};
use std::rc::Rc;

pub struct IntoTrimmedOutput<TokenType, TrimmedLexer> {
  pub digested: usize,
  pub errors: Vec<TokenType>,
  pub trimmed: TrimmedLexer,
}

pub struct TrimOutput<TokenType> {
  pub digested: usize,
  pub errors: Vec<TokenType>,
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static>
  LexerCore<Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  pub fn trim<'buffer, 'action_state, 'expect_text>(
    &self,
    buffer: &'buffer str,
    start: usize,
    mut action_state: &'action_state mut ActionState,
  ) -> TrimOutput<Rc<Token<'buffer, Kind, ErrorType>>>
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
      &mut action_state,
      &OUTPUT_HANDLER,
    );

    TrimOutput {
      digested: output.digested,
      errors: output.errors,
    }
  }
}
