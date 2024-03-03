use super::{common::Validator, StatelessLexer};
use crate::lexer::{
  options::ReLexContext, output::TrimOutput, stateless::common::OutputHandler, token::Token,
};

impl<Kind, ActionState, ErrorType> StatelessLexer<Kind, ActionState, ErrorType> {
  pub fn trim<'text, 'action_state, 'expect_text>(
    &self,
    text: &'text str,
    start: usize,
    mut action_state: &'action_state mut ActionState,
  ) -> TrimOutput<Token<'text, Kind, ErrorType>> {
    // use static to avoid allocation in each call
    static OUTPUT_HANDLER: OutputHandler = OutputHandler {
      update_lex_output: false,
      create_token: false,
    };

    let output = Self::execute_actions(
      &self.maybe_muted_head_map,
      ReLexContext::default(),
      move |_| Validator {
        // we already filtered actions, so never skip
        skip_before_exec: Box::new(|_| false),
        accept_after_exec: Box::new(|_, _| true),
      },
      text,
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
