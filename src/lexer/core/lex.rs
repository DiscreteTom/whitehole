pub mod expectation;
pub mod options;

use self::{expectation::Expectation, options::LexerCoreLexOptions};
use super::{
  common::{CallbackOutput, UpdateContext, Validator},
  LexerCore,
};
use crate::lexer::token::{Token, TokenKind};
use std::rc::Rc;

pub struct LexerCoreLexOutput<TokenType> {
  pub token: Option<TokenType>,
  pub digested: usize,
  pub errors: Vec<TokenType>,
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static>
  LexerCore<Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
{
  pub fn lex<'buffer, 'expect_text>(
    &mut self,
    buffer: &'buffer str,
    options: impl Into<LexerCoreLexOptions<'expect_text, Kind>>,
  ) -> LexerCoreLexOutput<Rc<Token<'buffer, Kind, ErrorType>>>
  where
    'buffer: 'expect_text,
  {
    let buffer: &str = buffer.into();
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
      move |input, output| {
        if output.muted {
          let token = if output.error.is_some() {
            Some(Self::output2token(input, output)) // record the error token
          } else {
            None
          };
          return CallbackOutput {
            update_ctx: UpdateContext::Yes { stop: false },
            token,
          };
        }

        let token = Self::output2token(input, output);
        CallbackOutput {
          update_ctx: UpdateContext::Yes { stop: true },
          token: Some(token),
        }
      },
    )
  }
}
