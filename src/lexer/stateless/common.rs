use super::{ActionHeadMap, StatelessLexer};
use crate::lexer::{
  action::{Action, ActionInput, ActionOutput},
  options::ReLexContext,
  output::LexOutput,
  token::{Range, Token},
};
use std::rc::Rc;

pub(crate) struct Validator<'validator, Kind, ActionState, ErrorType> {
  /// If return `true`, the action will be skipped.
  pub skip_before_exec: Box<dyn Fn(&Action<Kind, ActionState, ErrorType>) -> bool>,
  /// If return `true`, the action will be accepted.
  pub accept_after_exec: Box<
    dyn Fn(&ActionInput<ActionState>, &ActionOutput<Kind, Option<ErrorType>>) -> bool + 'validator, // make sure validator is not outlive the checker
  >,
}

/// This controls the behaviour of [`StatelessLexer::execute_actions`]
/// when an un-muted action is accepted.
pub(crate) struct UnMutedOutputHandler {
  /// If `true`, fields in [`LexOutput`] (like [`digested`](LexOutput::digested)) should be updated.
  pub update_lex_output: bool,
  /// If `true`, the [`LexOutput::token`] should be set.
  pub create_token: bool,
}

impl<Kind, ActionState, ErrorType> StatelessLexer<Kind, ActionState, ErrorType> {
  pub(crate) fn execute_actions<'text, 'validator, F>(
    head_map: &ActionHeadMap<Kind, ActionState, ErrorType>,
    fork: bool,
    re_lex: &ReLexContext,
    validator_factory: F,
    text: &'text str,
    start: usize,
    state: &mut ActionState,
    handler: &UnMutedOutputHandler,
  ) -> LexOutput<Token<'text, Kind, ErrorType>, ReLexContext>
  where
    F: Fn(&ActionInput<ActionState>) -> Validator<'validator, Kind, ActionState, ErrorType>,
  {
    let mut res = LexOutput {
      token: None, // should only be updated before return
      digested: 0,
      errors: Vec::new(),
      re_lex: None,
    };

    loop {
      // first, ensure rest is not empty
      // since maybe some token is muted in the last iteration which cause the rest is empty
      if start + res.digested >= text.len() {
        return res;
      }

      // all actions will reuse this action input
      // so we have to create it outside of the loop
      let mut input = ActionInput::new(text, start + res.digested, state);
      let validator = validator_factory(&input);
      let actions = head_map
        .known_map
        // TODO: maybe some day we can get a `&char` instead of a `char`
        .get(&(input.rest().chars().next().unwrap()))
        .unwrap_or(&head_map.unknown_fallback);
      let output = Self::traverse_actions(&mut input, actions, fork, re_lex, validator);

      match output {
        // all definition checked, no accepted action
        // but the digested and errors might be updated by the last iteration
        // so we have to return them
        None => return res,
        Some(TraverseActionsOutput { output, re_lex }) => {
          if output.error.is_some() {
            // error exists, we must create the token even muted
            // so we can collect the token in res.errors or res.token

            // backup values before output is consumed
            let muted = output.muted;
            let digested = output.digested;

            // create the error token
            let token = Self::create_token(&input, output);

            if muted {
              // don't emit token
              // push the token to errors
              // update state and continue
              // [[muted error tokens are also collected]]
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

            // set re-lex
            res.re_lex = re_lex;

            return res;
          }

          // else, no error, only create token if not muted

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
            res.token = Some(Self::create_token(&input, output));
          }

          // set re-lex
          res.re_lex = re_lex;

          return res;
        }
      }
    }
  }

  fn traverse_actions(
    input: &mut ActionInput<ActionState>,
    actions: &[Rc<Action<Kind, ActionState, ErrorType>>],
    fork: bool,
    re_lex: &ReLexContext,
    validator: Validator<Kind, ActionState, ErrorType>,
  ) -> Option<TraverseActionsOutput<Kind, ErrorType>> {
    for (i, action) in actions
      .iter()
      .enumerate()
      .skip(if input.start() == re_lex.start {
        re_lex.skip
      } else {
        0
      })
    {
      if let Some(output) = Self::try_execute_action(input, action, &validator) {
        return Some(TraverseActionsOutput {
          output,
          re_lex: if fork && i < actions.len() - 1 {
            // current action is not the last one
            // so the lex is re-lex-able
            Some(ReLexContext {
              skip: i + 1,
              start: input.start(),
            })
          } else {
            // fork is disabled or
            // current action is the last one
            // no next action to re-lex
            None
          },
        });
      }
    }
    // all actions are checked, no accepted action
    None
  }

  fn try_execute_action(
    input: &mut ActionInput<ActionState>,
    action: &Action<Kind, ActionState, ErrorType>,
    validator: &Validator<Kind, ActionState, ErrorType>,
  ) -> Option<ActionOutput<Kind, Option<ErrorType>>> {
    if (validator.skip_before_exec)(action) {
      return None;
    }

    action.exec(input).and_then(|output| {
      if (validator.accept_after_exec)(input, &output) {
        Some(output)
      } else {
        None
      }
    })
  }

  fn create_token<'text>(
    input: &ActionInput<'text, '_, ActionState>,
    output: ActionOutput<Kind, Option<ErrorType>>,
  ) -> Token<'text, Kind, ErrorType> {
    let range = Range {
      start: input.start(),
      end: input.start() + output.digested,
    };
    Token {
      kind: output.kind,
      content: &input.text()[range.start..range.end],
      range,
      error: output.error,
    }
  }
}

struct TraverseActionsOutput<Kind, ErrorType> {
  output: ActionOutput<Kind, Option<ErrorType>>,
  /// `None` if the current lexed action is the last one (no next action to re-lex).
  re_lex: Option<ReLexContext>,
}
