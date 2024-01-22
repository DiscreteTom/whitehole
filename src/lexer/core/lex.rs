pub mod expectation;
pub mod options;

use self::{expectation::Expectation, options::LexerCoreLexOptions};
use super::{common::Validator, LexerCore};
use crate::lexer::{
  core::common::OutputHandler,
  token::{Token, TokenKind},
};
use std::rc::Rc;

pub struct LexOutput<TokenType> {
  pub token: Option<TokenType>,
  pub digested: usize,
  pub errors: Vec<TokenType>,
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static>
  LexerCore<Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  pub fn lex<'buffer, 'expect_text>(
    &mut self,
    buffer: &'buffer str,
    options: impl Into<LexerCoreLexOptions<'expect_text, Kind>>,
  ) -> LexOutput<Rc<Token<'buffer, Kind, ErrorType>>>
  where
    'buffer: 'expect_text,
  {
    // use static to avoid allocation in each call
    static OUTPUT_HANDLER: OutputHandler = OutputHandler {
      update_lex_output: true,
      create_token: true,
    };

    let options: LexerCoreLexOptions<Kind> = options.into();
    let Expectation {
      kind: exp_kind,
      text: exp_text,
    } = options.expectation;
    let exp_kind = exp_kind.map(|kind| kind.id());

    Self::execute_actions(
      &self.actions,
      move |input| {
        let text_mismatch = exp_text.is_some_and(|text| input.rest().starts_with(text));
        Validator {
          skip_before_exec: Box::new(move |action| {
            action.never_muted()
              && (!(exp_kind.is_some_and(|kind| action.possible_kinds().contains(&kind)))
                || text_mismatch)
          }),
          accept_after_exec: Box::new(move |action, input, output| {
            output.muted
              || (!exp_kind.is_some_and(|kind| !action.possible_kinds().contains(&kind))
                && !exp_text.is_some_and(move |text| &input.rest()[..output.digested] != text))
          }),
        }
      },
      buffer,
      options.start,
      options.peek,
      &mut self.state,
      &OUTPUT_HANDLER,
    )
  }
}
