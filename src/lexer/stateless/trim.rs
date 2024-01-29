use super::{common::Validator, StatelessLexer};
use crate::lexer::{
  output::TrimOutput,
  stateless::common::OutputHandler,
  token::{Token, TokenKind},
};

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static>
  StatelessLexer<Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  pub fn trim<'buffer, 'action_state, 'expect_text>(
    &self,
    buffer: &'buffer str,
    start: usize,
    mut action_state: &'action_state mut ActionState,
  ) -> TrimOutput<Token<'buffer, Kind, ErrorType>>
  where
    'buffer: 'expect_text,
  {
    // use static to avoid allocation in each call
    static OUTPUT_HANDLER: OutputHandler = OutputHandler {
      update_lex_output: false,
      create_token: false,
    };

    let output = Self::execute_actions(
      &self.maybe_muted_actions,
      move |_| Validator {
        // we already filtered actions, so never skip
        skip_before_exec: Box::new(|_| false),
        accept_after_exec: Box::new(|_, _| true),
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
