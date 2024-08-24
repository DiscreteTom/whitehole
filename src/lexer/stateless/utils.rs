use super::head_map::RuntimeActions;
use crate::{
  lexer::{
    action::{ActionInput, ActionOutput},
    re_lex::ReLexContext,
    token::{Range, Token, TokenKindIdBinding},
  },
  utils::Accumulator,
};

/// Traverse all actions with a mutable input to find the first accepted action.
/// Return the output, the index of the accepted action and whether the action is muted.
/// If no accepted action, return [`None`].
pub(super) fn traverse_actions<'text, Kind, State, ErrorType>(
  mut input: ActionInput<&mut State>,
  actions: &RuntimeActions<Kind, State, ErrorType>,
  re_lex: &ReLexContext,
) -> Option<(
  ActionOutput<TokenKindIdBinding<Kind>, Option<ErrorType>>,
  usize,
  bool,
)> {
  for (i, exec) in actions
    .execs()
    .iter()
    .enumerate()
    .skip(if input.start() == re_lex.start {
      // SAFETY: it is ok that if `skip` is larger than `actions.len()`
      re_lex.skip
    } else {
      0
    })
  {
    if let Some(output) = (exec.raw)(&mut input) {
      debug_assert!(output.digested <= input.rest().len());
      // return once accepted action is found
      return Some((output, i, actions.muted()[i]));
    }
  }

  // no accepted action
  None
}

pub(super) fn update_state<ErrorType, ErrAcc: Accumulator<(ErrorType, Range)>>(
  output_digested: usize,
  error: Option<ErrorType>,
  start: usize,
  digested: &mut usize,
  errors: &mut ErrAcc,
) {
  // update digested, no matter the output is muted or not
  *digested += output_digested;

  // collect errors if any
  if let Some(err) = error {
    errors.update((err, create_range(start, output_digested)));
  }
}

/// Return the token if not muted, otherwise return [`None`].
#[inline]
pub(super) fn extract_token<Kind>(
  binding: TokenKindIdBinding<Kind>,
  output_digested: usize,
  muted: bool,
  start: usize,
) -> Option<Token<Kind>> {
  // if not muted, emit token
  (!muted).then(|| create_token(binding, start, output_digested))
}

#[inline]
const fn create_token<Kind>(
  binding: TokenKindIdBinding<Kind>,
  start: usize,
  digested: usize,
) -> Token<Kind> {
  Token {
    binding,
    range: create_range(start, digested),
  }
}

#[inline]
const fn create_range(start: usize, digested: usize) -> Range {
  Range {
    start,
    end: start + digested,
  }
}

macro_rules! break_loop_on_none {
  ($e:expr) => {
    match $e {
      Some(v) => v,
      None => break,
    }
  };
}
pub(super) use break_loop_on_none;

// TODO: add tests
