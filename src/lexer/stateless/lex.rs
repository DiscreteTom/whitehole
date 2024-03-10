use super::{common::Validator, StatelessLexer};
use crate::lexer::{
  expectation::Expectation,
  options::ReLexContext,
  output::LexOutput,
  stateless::common::OutputHandler,
  token::{Token, TokenKind},
};

pub struct StatelessLexOptions<'action_state, 'expect_text, Kind, ActionState> {
  pub action_state: &'action_state mut ActionState,
  pub start: usize,
  pub expectation: Expectation<'expect_text, Kind>,
  pub re_lex: Option<ReLexContext>, // TODO: rename to fork?
}

impl<'action_state, 'expect_text, Kind, ActionState>
  StatelessLexOptions<'action_state, 'expect_text, Kind, ActionState>
{
  pub fn state<'new_action_state, NewActionState>(
    self,
    action_state: &'new_action_state mut NewActionState,
  ) -> StatelessLexOptions<'new_action_state, 'expect_text, Kind, NewActionState> {
    StatelessLexOptions {
      action_state,
      start: self.start,
      expectation: self.expectation,
      re_lex: self.re_lex,
    }
  }
  pub fn start(mut self, start: usize) -> Self {
    self.start = start;
    self
  }
  pub fn expect(mut self, expectation: impl Into<Expectation<'expect_text, Kind>>) -> Self {
    self.expectation = expectation.into();
    self
  }
  pub fn re_lex(mut self, re_lex: ReLexContext) -> Self {
    self.re_lex = Some(re_lex);
    self
  }
}

pub struct StatelessLexOutput<TokenType, ActionState> {
  pub token: Option<TokenType>,
  pub digested: usize,
  pub errors: Vec<TokenType>,
  pub action_state: ActionState,
  // TODO: add re_lex
}

impl<Kind, ActionState, ErrorType> StatelessLexer<Kind, ActionState, ErrorType> {
  /// Lex from the start of the text, with the default action state.
  pub fn lex<'text>(
    &self,
    text: &'text str,
  ) -> StatelessLexOutput<Token<'text, Kind, ErrorType>, ActionState>
  where
    Kind: TokenKind<Kind>,
    ActionState: Default,
  {
    let mut action_state = ActionState::default();
    let output = self.lex_with(text, |o| o.state(&mut action_state));
    StatelessLexOutput {
      token: output.token,
      digested: output.digested,
      errors: output.errors,
      action_state,
    }
  }

  pub fn lex_default_with<'text, 'action_state, 'expect_text>(
    &self,
    text: &'text str,
    options_builder: impl FnOnce(
      StatelessLexOptions<'_, 'expect_text, Kind, ActionState>,
    )
      -> StatelessLexOptions<'action_state, 'expect_text, Kind, ActionState>,
  ) -> LexOutput<Token<'text, Kind, ErrorType>, ReLexContext>
  where
    Kind: TokenKind<Kind>,
    ActionState: Default + 'action_state,
  {
    let mut action_state = ActionState::default();
    self.lex_with(text, |o| options_builder(o.state(&mut action_state)))
  }

  pub fn lex_with<'text, 'action_state, 'expect_text, F>(
    &self,
    text: &'text str,
    options_builder: F,
  ) -> LexOutput<Token<'text, Kind, ErrorType>, ReLexContext>
  where
    Kind: TokenKind<Kind>,
    ActionState: 'action_state,
    F: FnOnce(
      StatelessLexOptions<'_, 'expect_text, Kind, ()>,
    ) -> StatelessLexOptions<'action_state, 'expect_text, Kind, ActionState>,
  {
    // use static to avoid allocation in each call
    static OUTPUT_HANDLER: OutputHandler = OutputHandler {
      update_lex_output: true,
      create_token: true,
    };

    let options = options_builder(StatelessLexOptions {
      action_state: &mut (),
      start: 0,
      expectation: Expectation::default(),
      re_lex: None,
    });
    let Expectation {
      kind: exp_kind,
      text: exp_text,
      ..
    } = options.expectation;
    let mut action_state = options.action_state;

    Self::execute_actions(
      exp_kind.map_or(
        // if no expected kind, use the head map with all actions
        &self.head_map,
        |kind| {
          self
            .kind_head_map
            .get(&kind)
            // this must be `Some`, unless the user set the wrong possible kinds for actions
            .expect("expected kind should exists in some action's possible kinds")
        },
      ),
      // the default ReLexContext will set `skip` and `action_index` to 0
      // which means this is not a re-lex
      options.re_lex.unwrap_or(ReLexContext::default()),
      move |input| {
        let text_mismatch = exp_text.is_some_and(|text| !input.rest().starts_with(text));
        Validator {
          // since we already filtered actions, we only need to skip actions
          // which are never muted and text mismatch
          skip_before_exec: Box::new(move |action| action.never_muted() && text_mismatch),
          accept_after_exec: Box::new(move |input, output| {
            output.muted
              || (
                // ensure expectation match.
                // we still need to check the kind after exec
                // because muted actions may yield unexpected kinds
                exp_kind.map_or(true, |kind| output.kind.id() == kind)
                  && exp_text.map_or(true, |text| &input.rest()[..output.digested] == text)
              )
          }),
        }
      },
      text,
      options.start,
      &mut action_state,
      &OUTPUT_HANDLER,
    )
  }
}
