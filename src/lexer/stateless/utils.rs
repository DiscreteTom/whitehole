use super::head_map::HeadMapActions;
use crate::{
  lexer::{
    action::{ActionInput, ActionOutput, GeneralAction},
    re_lex::{ReLexContext, ReLexableFactory},
    token::{Range, Token, TokenKindIdBinding},
  },
  utils::Accumulator,
};

/// Traverse all actions with an immutable input to find the first accepted action.
/// Return the output, the index of the accepted action and whether the action is muted.
/// If no accepted action, return [`None`].
/// If the action state is mutated during the traversal, return the new action state.
pub(super) fn traverse_actions<'text, Kind, State, ErrorType>(
  input: ActionInput<&State>,
  actions: &HeadMapActions<Kind, State, ErrorType>,
  re_lex: &ReLexContext,
) -> (
  Option<(
    ActionOutput<TokenKindIdBinding<Kind>, Option<ErrorType>>,
    usize,
    bool,
  )>,
  Option<State>,
)
where
  State: Clone,
{
  if let Some(res) = traverse_immutables(&input, actions, re_lex) {
    return (Some(res), None);
  }

  // if actions.rest is empty, prevent unnecessary cloning of the state
  if actions.rest().is_empty() {
    return (None, None);
  }

  // clone the state to construct mutable action input
  let mut state = input.state.clone();

  // we don't need re-lexable factory to clone the state here
  // because the `State` is already `Clone`
  // and it will always be cloned here

  (
    traverse_rest(&mut input.reload(&mut state), actions, re_lex),
    Some(state),
  )
}

/// Traverse all actions with a mutable input to find the first accepted action.
/// Return the output, the index of the accepted action and whether the action is muted.
/// If no accepted action, return [`None`].
pub(super) fn traverse_actions_mut<
  'text,
  Kind,
  State,
  ErrorType,
  ReLexableFactoryType: ReLexableFactory<'text, Kind, State, ErrorType>,
>(
  mut input: ActionInput<&mut State>,
  actions: &HeadMapActions<Kind, State, ErrorType>,
  re_lex: &ReLexContext,
  re_lexable_factory: &mut ReLexableFactoryType,
  peek: bool,
) -> Option<(
  ActionOutput<TokenKindIdBinding<Kind>, Option<ErrorType>>,
  usize,
  bool,
)> {
  if let Some(res) = traverse_immutables(&input.as_ref(), actions, re_lex) {
    return Some(res);
  }

  // if actions.rest is empty, prevent unnecessary cloning of the state
  if actions.rest().is_empty() {
    return None;
  }

  // when peek, we don't need to backup the action state
  // because the original state is not mutated,
  // so only backup when not peeking
  if !peek {
    re_lexable_factory.backup_state(input.state);
  }

  traverse_rest(&mut input, actions, re_lex)
}

fn traverse_immutables<Kind, State, ErrorType>(
  input: &ActionInput<&State>,
  actions: &HeadMapActions<Kind, State, ErrorType>,
  re_lex: &ReLexContext,
) -> Option<(
  ActionOutput<TokenKindIdBinding<Kind>, Option<ErrorType>>,
  usize,
  bool,
)> {
  for (i, action) in
    actions
      .immutables()
      .iter()
      .enumerate()
      .skip(if input.start() == re_lex.start {
        // SAFETY: it is ok that if `skip` is larger than `immutables.len()`
        re_lex.skip
      } else {
        0
      })
  {
    if let Some(output) = action.exec()(input) {
      // return once accepted action is found
      return Some((output, i, action.muted()));
    }
  }

  // no accepted action
  None
}

fn traverse_rest<'text, Kind, State, ErrorType>(
  input: &mut ActionInput<&mut State>,
  actions: &HeadMapActions<Kind, State, ErrorType>,
  re_lex: &ReLexContext,
) -> Option<(
  ActionOutput<TokenKindIdBinding<Kind>, Option<ErrorType>>,
  usize,
  bool,
)> {
  for (i, action) in actions
    .rest()
    .iter()
    .enumerate()
    .skip(if input.start() == re_lex.start {
      // prevent subtraction overflow, e.g. skip is 0
      re_lex.skip.saturating_sub(actions.immutables().len())
    } else {
      0
    })
  {
    if let Some(output) = match action {
      GeneralAction::Immutable(action) => action.exec()(&input.as_ref()),
      GeneralAction::Mutable(action) => action.exec()(input),
    } {
      // return once accepted action is found
      return Some((output, i + actions.immutables().len(), action.muted()));
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
