use super::{common::Validator, StatelessLexer};
use crate::lexer::{
  expectation::Expectation,
  output::{LexOutput, ReLexActionIndex},
  stateless::common::OutputHandler,
  token::{Token, TokenKind},
};

pub struct StatelessLexOptions<'action_state, 'expect, Kind, ActionState: Clone + Default> {
  pub start: usize,
  pub action_state: &'action_state mut ActionState,
  pub expectation: Expectation<'expect, Kind>,
  pub from_index: ReLexActionIndex,
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
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  pub fn lex<'buffer>(
    &self,
    buffer: &'buffer str,
  ) -> StatelessLexOutput<Token<'buffer, Kind, ErrorType>, ActionState> {
    let mut action_state = ActionState::default();
    let output = self.lex_with(
      buffer,
      StatelessLexOptions {
        start: 0,
        expectation: Expectation::default(),
        action_state: &mut action_state,
        from_index: ReLexActionIndex(0),
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
  ) -> LexOutput<Token<'buffer, Kind, ErrorType>, ReLexActionIndex>
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
      ..
    } = options.expectation;
    let mut action_state = options.action_state;

    Self::execute_actions(
      exp_kind.map_or(&self.head_map, |kind| {
        self.kind_head_map.get(&kind).unwrap_or(&self.head_map)
      }),
      options.from_index,
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
      buffer,
      options.start,
      &mut action_state,
      &OUTPUT_HANDLER,
    )
  }
}
