use super::{common::Validator, StatelessLexer};
use crate::lexer::{
  options::ReLexContext, output::TrimOutput, stateless::common::OutputHandler, token::Token,
};

impl<Kind, ActionState, ErrorType> StatelessLexer<Kind, ActionState, ErrorType> {
  /// Trim from the start of the text with the default action state.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)] enum MyKind { A }
  /// # let stateless = LexerBuilder::<MyKind>::new().define(MyKind::A, exact("1")).build_stateless();
  /// stateless.trim("123");
  /// ```
  pub fn trim<'text>(
    &self,
    text: &'text str,
  ) -> (TrimOutput<Token<'text, Kind, ErrorType>>, ActionState)
  where
    ActionState: Default,
  {
    let mut action_state = ActionState::default();
    (self.trim_with(text, 0, &mut action_state), action_state)
  }

  /// Trim the text with the given start position and action state.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)] enum MyKind { A }
  /// # let stateless = LexerBuilder::<MyKind>::new().define(MyKind::A, exact("1")).build_stateless();
  /// stateless.trim_with("123", 0, &mut ());
  /// ```
  pub fn trim_with<'text>(
    &self,
    text: &'text str,
    start: usize,
    action_state: &mut ActionState,
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
      action_state,
      &OUTPUT_HANDLER,
    );

    TrimOutput {
      digested: output.digested,
      errors: output.errors,
    }
  }
}
