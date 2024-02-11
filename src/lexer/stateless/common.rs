use super::StatelessLexer;
use crate::lexer::{
  action::{input::ActionInput, output::ActionOutput, Action},
  output::LexOutput,
  token::{Range, Token, TokenKind},
};
use std::rc::Rc;

pub struct Validator<'validator, Kind: 'static, ActionState: 'static, ErrorType: 'static> {
  /// If return `true`, the action will be skipped.
  pub skip_before_exec: Box<dyn Fn(&Action<Kind, ActionState, ErrorType>) -> bool>,
  /// If return `true`, the action will be accepted.
  pub accept_after_exec: Box<
    dyn Fn(&ActionInput<ActionState>, &ActionOutput<Kind, ErrorType>) -> bool + 'validator, // make sure validator is not outlive the checker
  >,
}

/// OutputHandler controls the behaviour of `execute_actions`
/// when an un-muted action is accepted.
pub struct OutputHandler {
  /// If `true`, fields in `LexerCoreLexOutput` (like `digested`) should be updated.
  pub update_lex_output: bool,
  /// If `true`, the `LexerCoreLexOutput` should have a token created by the `ActionOutput`.
  pub create_token: bool,
}

impl<'input, 'buffer, 'state, Kind, ActionState, ErrorType>
  StatelessLexer<Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  pub fn execute_actions<'validator, F>(
    actions: &[Rc<Action<Kind, ActionState, ErrorType>>],
    validator_factory: F,
    buffer: &'buffer str,
    start: usize,
    state: &'state mut ActionState,
    handler: &OutputHandler,
  ) -> LexOutput<Token<'buffer, Kind, ErrorType>>
  where
    F: Fn(&ActionInput<ActionState>) -> Validator<'validator, Kind, ActionState, ErrorType>,
  {
    let mut res = LexOutput {
      token: None, // should only be updated before return
      digested: 0,
      errors: Vec::new(),
    };

    loop {
      // first, ensure rest is not empty
      // since maybe some token is muted in the last iteration which cause the rest is empty
      if start + res.digested >= buffer.len() {
        return res;
      }

      // all actions will reuse this action input to reuse lazy values
      // so we have to create it outside of the loop
      let mut input = ActionInput::new(buffer, start + res.digested, state);
      let validator = validator_factory(&input);
      let output = Self::traverse_actions(&mut input, actions, validator);

      match output {
        // all definition checked, no accepted action
        // but the digested and errors might be updated by the last iteration
        // so we have to return them
        None => return res,
        Some(output) => {
          if output.error.is_some() {
            // copy values before output is consumed
            let muted = output.muted;
            let digested = output.digested;

            // create the error token
            let token = Self::output2token(&input, output);

            if muted {
              // don't emit token
              // push the token to errors
              // update state and continue
              res.errors.push(token);
              res.digested += digested;
              continue;
            }

            // else, not muted, check output handler
            if handler.update_lex_output {
              res.digested += digested;
            }
            if handler.create_token {
              res.token = Some(token);
            }
            return res;
          } else {
            // else, no error

            if output.muted {
              // don't emit token
              // just update state and continue
              res.digested += output.digested;
              continue;
            }

            // else, not muted, check output handler
            if handler.update_lex_output {
              res.digested += output.digested;
            }
            if handler.create_token {
              res.token = Some(Self::output2token(&input, output));
            }
            return res;
          }
        }
      }
    }
  }

  fn traverse_actions(
    input: &mut ActionInput<'buffer, 'state, ActionState>,
    actions: &[Rc<Action<Kind, ActionState, ErrorType>>],
    validator: Validator<Kind, ActionState, ErrorType>,
  ) -> Option<ActionOutput<Kind, ErrorType>> {
    for action in actions {
      if let Some(output) = Self::try_execute_action(input, action, &validator) {
        return Some(output);
      }
    }
    None
  }

  fn try_execute_action(
    input: &'input mut ActionInput<'buffer, 'state, ActionState>,
    action: &Action<Kind, ActionState, ErrorType>,
    validator: &Validator<Kind, ActionState, ErrorType>,
  ) -> Option<ActionOutput<Kind, ErrorType>> {
    if (validator.skip_before_exec)(action) {
      return None;
    }

    let output = action.exec(input);

    if let Some(output) = output {
      if (validator.accept_after_exec)(input, &output) {
        return Some(output);
      }
    }

    // output is None, action is rejected
    None
  }

  pub fn output2token(
    input: &ActionInput<'buffer, '_, ActionState>,
    output: ActionOutput<Kind, ErrorType>,
  ) -> Token<'buffer, Kind, ErrorType> {
    Token {
      kind: output.kind,
      buffer: input.buffer(),
      range: Range {
        start: input.start(),
        end: input.start() + output.digested,
      },
      error: output.error,
    }
  }
}
