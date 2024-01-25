pub mod expectation;
pub mod options;

use self::{expectation::Expectation, options::StatelessLexOptions};
use super::{common::Validator, StatelessLexer};
use crate::lexer::{
  stateless::common::OutputHandler,
  token::{Token, TokenKind},
};
use std::rc::Rc;

// TODO: move
pub struct LexOutput<TokenType> {
  pub token: Option<TokenType>,
  pub digested: usize,
  pub errors: Vec<TokenType>,
}

pub struct PeekOutput<TokenType, ActionState> {
  pub token: Option<TokenType>,
  pub digested: usize,
  pub errors: Vec<TokenType>,
  pub action_state: ActionState,
}

pub struct LexAllOutput<TokenType> {
  pub tokens: Vec<TokenType>,
  pub digested: usize,
  pub errors: Vec<TokenType>,
}

pub struct StatelessLexOutput<TokenType, ActionState> {
  pub token: Option<TokenType>,
  pub digested: usize,
  pub errors: Vec<TokenType>,
  pub action_state: ActionState,
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static>
  StatelessLexer<Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  pub fn lex<'buffer>(
    &self,
    buffer: &'buffer str,
  ) -> StatelessLexOutput<Rc<Token<'buffer, Kind, ErrorType>>, ActionState> {
    let mut action_state = ActionState::default();
    let output = self.lex_with(
      buffer,
      StatelessLexOptions {
        start: 0,
        expectation: Expectation::default(),
        action_state: &mut action_state,
      },
    );
    StatelessLexOutput {
      token: output.token,
      digested: output.digested,
      errors: output.errors,
      action_state,
    }
  }

  pub fn lex_with<'buffer, 'action_state, 'expect_text>(
    &self,
    buffer: &'buffer str,
    options: impl Into<StatelessLexOptions<'action_state, 'expect_text, Kind, ActionState>>,
  ) -> LexOutput<Rc<Token<'buffer, Kind, ErrorType>>>
  where
    'buffer: 'expect_text,
  {
    // use static to avoid allocation in each call
    static OUTPUT_HANDLER: OutputHandler = OutputHandler {
      update_lex_output: true,
      create_token: true,
    };

    let options: StatelessLexOptions<Kind, ActionState> = options.into();
    let Expectation {
      kind: exp_kind,
      text: exp_text,
    } = options.expectation;
    let exp_kind = exp_kind.map(|kind| kind.id());
    let mut action_state = options.action_state;

    Self::execute_actions(
      &self.actions,
      move |input| {
        let text_mismatch = exp_text.is_some_and(|text| !input.rest().starts_with(text));
        Validator {
          skip_before_exec: Box::new(move |action| {
            action.never_muted()
              && ((exp_kind.is_some_and(|kind| !action.possible_kinds().contains(&kind)))
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
      &mut action_state,
      &OUTPUT_HANDLER,
    )
  }
}
