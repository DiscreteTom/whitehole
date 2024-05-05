use super::{HeadMap, StatelessLexer};
use crate::lexer::{
  action::{Action, ActionInput, ActionOutput},
  options::ReLexContext,
  output::LexOutput,
  token::{Range, Token, TokenKindIdProvider},
};
use std::rc::Rc;

impl<Kind, ActionState, ErrorType> StatelessLexer<Kind, ActionState, ErrorType> {
  pub(super) fn execute_actions<'text, 'head_map>(
    head_map_getter: impl Fn(
      &ActionInput<ActionState>,
    ) -> &'head_map HeadMap<Kind, ActionState, ErrorType>,
    fork: bool,
    re_lex: &ReLexContext,
    text: &'text str,
    start: usize,
    state: &mut ActionState,
  ) -> LexOutput<Token<'text, Kind, ErrorType>, ReLexContext>
  where
    Kind: TokenKindIdProvider<Kind> + 'static,
    ActionState: 'head_map,
    ErrorType: 'head_map,
  {
    let mut res = LexOutput {
      digested: 0,        // might be updated during the loop
      errors: Vec::new(), // might be updated during the loop
      token: None,        // should only be set before return
      re_lex: None,       // should only be set before return
    };

    loop {
      // all actions will reuse this action input in this iteration
      let mut input = match ActionInput::new(text, start + res.digested, state) {
        None => {
          // maybe some token is muted in the last iteration which cause the rest is empty
          // but the `res.digested` might be updated by the last iteration
          // so we have to return the result
          return res;
        }
        Some(input) => input,
      };

      let head_map = head_map_getter(&input);
      let actions = head_map
        .known_map()
        // TODO: maybe some day we can get a `&char` instead of a `char`
        .get(&(input.rest().chars().next().unwrap()))
        .unwrap_or(head_map.unknown_fallback());

      match Self::traverse_actions(&mut input, actions, re_lex) {
        // all definition checked, no accepted action
        // but the digested and errors might be updated by the last iteration
        // so we have to return them
        None => return res,
        Some((output, action_index)) => {
          // update digested, no matter the output is muted or not
          res.digested += output.digested;

          if output.error.is_some() {
            // error exists, we must create the token even muted
            // so we can collect the token in `res.errors` or `res.token`

            // create the error token
            let token = Self::create_token(&input, output);

            if actions[action_index].muted() {
              // don't emit token
              // collect errors and continue
              // [[muted error tokens are also collected]]
              res.errors.push(token);
              continue;
            }

            // else, not muted
            // don't push token to errors, set the res.token
            res.token = Some(token);
            res.re_lex = Self::create_re_lex_context(fork, &input, actions, action_index);

            return res;
          }

          // else, no error, only create token if not muted

          if actions[action_index].muted() {
            // don't emit token
            continue;
          }

          // else, not muted
          res.token = Some(Self::create_token(&input, output));
          res.re_lex = Self::create_re_lex_context(fork, &input, actions, action_index);

          return res;
        }
      }
    }
  }

  /// Traverse all actions to find the first accepted action.
  /// Return the output and the index of the accepted action.
  /// If no accepted action, return `None`.
  fn traverse_actions(
    input: &mut ActionInput<ActionState>,
    actions: &[Rc<Action<Kind, ActionState, ErrorType>>],
    re_lex: &ReLexContext,
  ) -> Option<(ActionOutput<Kind, Option<ErrorType>>, usize)>
  where
    Kind: TokenKindIdProvider<Kind>,
  {
    for (i, action) in actions
      .iter()
      .enumerate()
      .skip(if input.start() == re_lex.start {
        re_lex.skip
      } else {
        0
      })
    {
      if let Some(output) = action.exec(input) {
        return Some((output, i));
      }
    }
    // all actions are checked, no accepted action
    None
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

  fn create_re_lex_context(
    fork: bool,
    input: &ActionInput<ActionState>,
    actions: &[Rc<Action<Kind, ActionState, ErrorType>>],
    action_index: usize,
  ) -> Option<ReLexContext> {
    if fork && action_index < actions.len() - 1 {
      // current action is not the last one
      // so the lex is re-lex-able
      Some(ReLexContext {
        skip: action_index + 1, // index + 1 is the count of actions to skip
        start: input.start(),
      })
    } else {
      // fork is disabled or
      // current action is the last one
      // no next action to re-lex
      None
    }
  }
}
