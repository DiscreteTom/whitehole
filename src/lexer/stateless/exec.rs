use super::{head_map::HeadMapActions, output::StatelessOutputFactory, HeadMap, StatelessLexer};
use crate::{
  lexer::{
    action::{ActionInput, ActionOutput, GeneralAction},
    re_lex::{ReLexContext, ReLexableFactory},
    token::{Range, Token, TokenKindIdProvider},
  },
  utils::Accumulator,
};

impl<Kind, ActionState, ErrorType> StatelessLexer<Kind, ActionState, ErrorType> {
  pub(super) fn execute_actions<
    'text,
    'head_map,
    ErrAcc: Accumulator<(ErrorType, Range)>,
    ReLexableFactoryType: ReLexableFactory<'text, Kind, ActionState, ErrorType>,
    StatelessOutputType: StatelessOutputFactory<Token<Kind>, ErrAcc, ReLexableFactoryType::StatelessReLexableType>,
  >(
    head_map_getter: impl Fn(&str) -> &'head_map HeadMap<Kind, ActionState, ErrorType>,
    re_lex: &ReLexContext,
    text: &'text str,
    start: usize,
    state: &ActionState,
    mut re_lexable_factory: ReLexableFactoryType,
    mut res: StatelessOutputType,
  ) -> (StatelessOutputType::Target, Option<ActionState>)
  where
    Kind: TokenKindIdProvider<TokenKind = Kind> + 'static,
    ActionState: Clone + 'head_map,
    ErrorType: 'head_map,
  {
    // since the provided action state is an immutable reference,
    // if the action state needs to be mutated, it will be cloned and returned
    let mut new_state = None;

    while let Some((input, actions)) =
      prepare_input(text, start + res.digested(), state, &head_map_getter)
    {
      let (output, state) = traverse_actions(&input, actions, re_lex, &mut re_lexable_factory);
      new_state = state;

      let target = match process_output(output, input.start(), &mut res) {
        ProcessResult::Continue => {
          // if the action state is mutated, use it
          if let Some(state) = &mut new_state {
            return (
              Self::execute_actions_mut(
                head_map_getter,
                re_lex,
                text,
                start + res.digested(),
                state,
                re_lexable_factory,
                res,
              ),
              new_state,
            );
          }
          continue;
        }
        ProcessResult::Return => res.emit(),
        ProcessResult::Token((token, action_index)) => res.emit_with_token(
          token,
          re_lexable_factory.into_stateless_re_lexable(input.start(), actions.len(), action_index),
        ),
      };
      return (target, new_state);
    }

    return (res.emit(), new_state);
  }

  pub(super) fn execute_actions_mut<
    'text,
    'head_map,
    ErrAcc: Accumulator<(ErrorType, Range)>,
    ReLexableFactoryType: ReLexableFactory<'text, Kind, ActionState, ErrorType>,
    StatelessOutputType: StatelessOutputFactory<Token<Kind>, ErrAcc, ReLexableFactoryType::StatelessReLexableType>,
  >(
    head_map_getter: impl Fn(&str) -> &'head_map HeadMap<Kind, ActionState, ErrorType>,
    re_lex: &ReLexContext,
    text: &'text str,
    start: usize,
    state: &mut ActionState,
    mut re_lexable_factory: ReLexableFactoryType,
    mut res: StatelessOutputType,
  ) -> StatelessOutputType::Target
  where
    Kind: TokenKindIdProvider<TokenKind = Kind> + 'static,
    ActionState: 'head_map,
    ErrorType: 'head_map,
  {
    while let Some((mut input, actions)) =
      prepare_input(text, start + res.digested(), &mut *state, &head_map_getter)
    {
      let output = traverse_actions_mut(&mut input, actions, re_lex, &mut re_lexable_factory);

      let target = match process_output(output, input.start(), &mut res) {
        ProcessResult::Continue => continue,
        ProcessResult::Return => res.emit(),
        ProcessResult::Token((token, action_index)) => res.emit_with_token(
          token,
          re_lexable_factory.into_stateless_re_lexable(input.start(), actions.len(), action_index),
        ),
      };
      return target;
    }

    return res.emit();
  }
}

enum ProcessResult<TokenType> {
  Continue,
  Return,
  Token((TokenType, usize)),
}

fn process_output<
  'text,
  Kind: 'static,
  ErrorType,
  ErrAcc: Accumulator<(ErrorType, Range)>,
  StatelessReLexableType,
  StatelessOutputType: StatelessOutputFactory<Token<Kind>, ErrAcc, StatelessReLexableType>,
>(
  output: Option<(ActionOutput<Kind, Option<ErrorType>>, usize, bool)>,
  start: usize,
  res: &mut StatelessOutputType,
) -> ProcessResult<Token<Kind>> {
  match output {
    // all definition checked, no accepted action
    // but the digested and errors might be updated by the last iteration
    // so we have to return them
    None => return ProcessResult::Return,
    Some((output, action_index, muted)) => {
      // update digested, no matter the output is muted or not
      res.digest(output.digested);

      // collect errors
      if let Some(err) = output.error {
        res
          .errors()
          .update((err, create_range(start, output.digested)));
      }

      // don't emit token if muted
      if muted {
        return ProcessResult::Continue;
      }

      // not muted, emit token
      return ProcessResult::Token((
        create_token(output.kind, start, output.digested),
        action_index,
      ));
    }
  }
}

#[inline]
fn prepare_input<
  'text,
  'head_map: 'text,
  Kind: 'static,
  ActionState: 'head_map,
  ActionStateRef,
  ErrorType: 'head_map,
>(
  text: &'text str,
  start: usize,
  state: ActionStateRef,
  head_map_getter: &impl Fn(&str) -> &'head_map HeadMap<Kind, ActionState, ErrorType>,
) -> Option<(
  ActionInput<'text, ActionStateRef>,
  &'head_map HeadMapActions<Kind, ActionState, ErrorType>,
)> {
  ActionInput::new(text, start, state).map(|input| {
    let head_map = head_map_getter(input.rest());
    let actions = head_map.get(input.next());

    (input, actions)
  })
}

/// Traverse all actions to find the first accepted action.
/// Return the output and the index of the accepted action.
/// If no accepted action, return `None`.
fn traverse_actions<
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
fn traverse_actions_mut<
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
      GeneralAction::Mutable(action) => {
        re_lexable_factory.before_mutate_action_state(input.state);
        action.exec()(input)
      }
    } {
      // return once accepted action is found
      return Some((output, i + actions.immutables().len(), action.muted()));
    }
  }

  // no accepted action
  None
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
