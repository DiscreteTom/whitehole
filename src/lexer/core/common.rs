use super::{lex::LexerCoreLexOutput, LexerCore};
use crate::lexer::{
  action::{input::ActionInput, output::ActionOutput, Action},
  token::Token,
};
use std::rc::Rc;

pub struct Validator<'validator, Kind: 'static, ActionState: 'static, ErrorType: 'static> {
  /// If return `true`, the action will be skipped.
  pub skip_before_exec: Box<dyn Fn(&Action<Kind, ActionState, ErrorType>) -> bool>,
  /// If return `true`, the action will be accepted.
  pub accept_after_exec: Box<
    dyn Fn(
        &Action<Kind, ActionState, ErrorType>,
        &ActionInput<ActionState>,
        &ActionOutput<Kind, ErrorType>,
      ) -> bool
      + 'validator, // make sure validator is not outlive the checker
  >,
}

pub enum UpdateContext {
  Yes { stop: bool },
  No,
}

pub struct CallbackOutput<'buffer, Kind, ErrorType> {
  // TODO: add comments, optimize name
  pub update_ctx: UpdateContext,
  pub token: Option<Token<'buffer, Kind, ErrorType>>,
}

impl<'input, 'buffer, 'state, Kind, ActionState, ErrorType> LexerCore<Kind, ActionState, ErrorType>
where
  // TODO: remove these?
  'buffer: 'input,
  'state: 'input,
{
  pub fn execute_actions<'validator, F, C>(
    actions: &[Action<Kind, ActionState, ErrorType>],
    validator_factory: F,
    buffer: &'buffer str,
    start: usize,
    peek: bool,
    state: &'state mut ActionState,
    callback: C,
  ) -> LexerCoreLexOutput<Rc<Token<'buffer, Kind, ErrorType>>>
  where
    F: Fn(&ActionInput<ActionState>) -> Validator<'validator, Kind, ActionState, ErrorType>,
    C: Fn(
      &ActionInput<'buffer, '_, ActionState>,
      ActionOutput<Kind, ErrorType>,
    ) -> CallbackOutput<'buffer, Kind, ErrorType>,
  {
    let mut res = LexerCoreLexOutput {
      token: None,
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
      let mut input = ActionInput::new(buffer, start + res.digested, state, peek);
      let validator = validator_factory(&input);
      let output = Self::traverse_actions(&mut input, actions, validator);

      if let Some(output) = output {
        let digested = output.digested;
        let cb_res = callback(&input, output);
        let token = cb_res.token.map(Rc::new);

        // collect errors
        if let Some(token) = &token {
          if token.error.is_some() {
            res.errors.push(token.clone());
          }
        }

        // update digested
        if let UpdateContext::Yes { stop: _ } = cb_res.update_ctx {
          res.digested += digested;
        }

        // stop lexing
        // TODO: optimize code
        if let UpdateContext::No = cb_res.update_ctx {
          // stop lexing
          res.token = token;
          return res;
        }
        if let UpdateContext::Yes { stop } = cb_res.update_ctx {
          if stop {
            // stop lexing
            res.token = token;
            return res;
          }
        }
      } else {
        // all definition checked, no accepted action
        // but the digested and errors might be updated by the last iteration
        // so we have to return them
        return res;
      }
    }
  }

  fn traverse_actions(
    input: &mut ActionInput<'buffer, 'state, ActionState>,
    actions: &[Action<Kind, ActionState, ErrorType>],
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
    if !(validator.skip_before_exec)(action) {
      return None;
    }

    let output = action.exec(input);

    if let Some(output) = output {
      if (validator.accept_after_exec)(action, input, &output) {
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
      start: input.start(),
      end: input.start() + output.digested,
      error: output.error,
    }
  }
}
