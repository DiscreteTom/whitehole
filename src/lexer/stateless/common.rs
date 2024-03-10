use super::{ActionHeadMap, StatelessLexer};
use crate::lexer::{
  action::{Action, ActionInput, ActionOutput},
  expectation::Expectation,
  options::ReLexContext,
  output::LexOutput,
  token::{Range, Token, TokenKind},
};
use std::rc::Rc;

impl<Kind, ActionState, ErrorType> StatelessLexer<Kind, ActionState, ErrorType> {
  pub(crate) fn execute_actions<'text, 'expect_text>(
    head_map: &ActionHeadMap<Kind, ActionState, ErrorType>,
    fork: bool,
    re_lex: &ReLexContext,
    text: &'text str,
    start: usize,
    state: &mut ActionState,
    expectation: &Expectation<'expect_text, Kind>,
  ) -> LexOutput<Token<'text, Kind, ErrorType>, ReLexContext>
  where
    Kind: TokenKind<Kind>,
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
      let text_mismatch = expectation
        .text
        .is_some_and(|text| !input.rest().starts_with(text));
      let actions = head_map
        .known_map
        // TODO: maybe some day we can get a `&char` instead of a `char`
        .get(&(input.rest().chars().next().unwrap()))
        .unwrap_or(&head_map.unknown_fallback);
      let output = Self::traverse_actions(
        &mut input,
        actions,
        fork,
        re_lex,
        text_mismatch,
        &expectation,
      );

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

            // else, not muted
            res.digested += digested;
            res.token = Some(token);

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

          // else, not muted
          res.digested += output.digested;
          res.token = Some(Self::create_token(&input, output));

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
    text_mismatch: bool,
    expectation: &Expectation<Kind>,
  ) -> Option<TraverseActionsOutput<Kind, ErrorType>>
  where
    Kind: TokenKind<Kind>,
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
      if let Some(output) = Self::try_execute_action(input, action, text_mismatch, expectation) {
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
    text_mismatch: bool,
    expectation: &Expectation<Kind>,
  ) -> Option<ActionOutput<Kind, Option<ErrorType>>>
  where
    Kind: TokenKind<Kind>,
  {
    // TODO: pre-calc and cache never muted actions
    if text_mismatch && action.never_muted() {
      // text mismatch, only muted actions should be executed
      // so we skip never muted actions
      return None;
    }

    action.exec(input).and_then(|output| {
      if
      // muted output is always accepted regardless of the expectation
      output.muted
        || (
          // ensure expectation match.
          // we still need to check the kind after exec
          // because maybe_muted actions may yield unexpected kinds and actually not muted
          expectation.kind.map_or(true, |kind| output.kind.id() == kind)
          // same as the text, maybe_muted actions may accept unexpected text and actually not muted
            && expectation.text.map_or(true, |text| &input.rest()[..output.digested] == text)
        )
      {
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
