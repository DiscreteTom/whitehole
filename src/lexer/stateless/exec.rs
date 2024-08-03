use super::head_map::HeadMapActions;
use crate::{
  lexer::{
    action::{ActionInput, ActionOutput, GeneralAction},
    re_lex::{ReLexContext, ReLexableFactory},
    token::{Range, Token, TokenKindIdProvider},
  },
  utils::Accumulator,
};

/// Traverse all actions to find the first accepted action.
/// Return the output and the index of the accepted action.
/// If no accepted action, return `None`.
pub(super) fn traverse_actions<
  'text,
  Kind,
  ActionState,
  ErrorType,
  ReLexableFactoryType: ReLexableFactory<'text, Kind, ActionState, ErrorType>,
>(
  input: &ActionInput<&ActionState>,
  actions: &HeadMapActions<Kind, ActionState, ErrorType>,
  re_lex: &ReLexContext,
  re_lexable_factory: &mut ReLexableFactoryType,
) -> (
  Option<(ActionOutput<Kind, Option<ErrorType>>, usize, bool)>,
  Option<ActionState>,
)
where
  Kind: TokenKindIdProvider<TokenKind = Kind>,
  ActionState: Clone,
{
  if let Some(res) = traverse_immutables(input, actions, re_lex) {
    return (Some(res), None);
  }

  // prevent unnecessary clone if there is no mutable actions
  if actions.rest().len() == 0 {
    return (None, None);
  }

  // clone the state to construct mutable action input
  let mut state = input.state.clone();

  (
    traverse_rest(
      &mut input.clone_with(&mut state),
      actions,
      re_lex,
      re_lexable_factory,
    ),
    Some(state),
  )
}

/// Traverse all actions to find the first accepted action.
/// Return the output and the index of the accepted action.
/// If no accepted action, return `None`.
pub(super) fn traverse_actions_mut<
  'text,
  Kind,
  ActionState,
  ErrorType,
  ReLexableFactoryType: ReLexableFactory<'text, Kind, ActionState, ErrorType>,
>(
  input: &mut ActionInput<&mut ActionState>,
  actions: &HeadMapActions<Kind, ActionState, ErrorType>,
  re_lex: &ReLexContext,
  re_lexable_factory: &mut ReLexableFactoryType,
) -> Option<(ActionOutput<Kind, Option<ErrorType>>, usize, bool)>
where
  Kind: TokenKindIdProvider<TokenKind = Kind>,
{
  if let Some(res) = traverse_immutables(&input.as_ref(), actions, re_lex) {
    return Some(res);
  }

  traverse_rest(input, actions, re_lex, re_lexable_factory)
}

fn traverse_immutables<Kind, ActionState, ErrorType>(
  input: &ActionInput<&ActionState>,
  actions: &HeadMapActions<Kind, ActionState, ErrorType>,
  re_lex: &ReLexContext,
) -> Option<(ActionOutput<Kind, Option<ErrorType>>, usize, bool)> {
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

fn traverse_rest<
  'text,
  Kind,
  ActionState,
  ErrorType,
  ReLexableFactoryType: ReLexableFactory<'text, Kind, ActionState, ErrorType>,
>(
  input: &mut ActionInput<&mut ActionState>,
  actions: &HeadMapActions<Kind, ActionState, ErrorType>,
  re_lex: &ReLexContext,
  re_lexable_factory: &mut ReLexableFactoryType,
) -> Option<(ActionOutput<Kind, Option<ErrorType>>, usize, bool)> {
  // prevent unnecessary clone of the action state
  if actions.rest().len() == 0 {
    return None;
  }

  // actions.rest is not empty, the first one must be a mutable action
  // so we should backup the action state here
  re_lexable_factory.before_mutate_action_state(input.state);

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
  kind: Kind,
  output_digested: usize,
  muted: bool,
  start: usize,
) -> Option<Token<Kind>> {
  // if not muted, emit token
  (!muted).then(|| create_token(kind, start, output_digested))
}

#[inline]
const fn create_token<Kind>(kind: Kind, start: usize, digested: usize) -> Token<Kind> {
  Token {
    kind,
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

// TODO: add tests
