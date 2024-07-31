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
    loop {
      // all actions will reuse this action input in this iteration
      let input = match ActionInput::new(text, start + res.digested(), state) {
        None => {
          // maybe some token is muted in previous iterations which cause the rest is empty
          // but the `res.digested` might be updated by previous iterations
          // so we have to return the result instead of a `None`
          return (res.emit(), None); // TODO: this shouldn't be None, fix this
        }
        Some(input) => input,
      };

      let head_map = head_map_getter(input.rest());
      let actions = head_map
        .known_map()
        .get(&input.next())
        .unwrap_or(head_map.unknown_fallback());

      match Self::traverse_actions(&input, actions, re_lex, &mut re_lexable_factory) {
        // all definition checked, no accepted action
        // but the digested and errors might be updated by the last iteration
        // so we have to return them
        (None, state) => return (res.emit(), state),
        (Some((output, action_index, muted)), state) => {
          // update digested, no matter the output is muted or not
          res.digest(output.digested);

          match output.error {
            Some(err) => {
              if muted {
                // err but muted, collect errors and continue
                res
                  .errors()
                  .update((err, create_range(input.start(), output.digested)));
                continue;
              } else {
                // err and not muted, collect error and emit token
                let token = create_token(output.kind, input.start(), output.digested);
                res.errors().update((err, token.range.clone()));
                return (
                  res.emit_with_token(
                    token,
                    re_lexable_factory.into_stateless_re_lexable(
                      input.start(),
                      actions.len(),
                      action_index,
                    ),
                  ),
                  state,
                );
              }
            }
            None => {
              // else, no error

              // don't emit token if muted
              if muted {
                continue;
              }

              // not muted, emit token
              return (
                res.emit_with_token(
                  create_token(output.kind, input.start(), output.digested),
                  re_lexable_factory.into_stateless_re_lexable(
                    input.start(),
                    actions.len(),
                    action_index,
                  ),
                ),
                state,
              );
            }
          }
        }
      }
    }
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
    loop {
      // all actions will reuse this action input in this iteration
      let mut input = match ActionInput::new(text, start + res.digested(), &mut *state) {
        None => {
          // maybe some token is muted in previous iterations which cause the rest is empty
          // but the `res.digested` might be updated by previous iterations
          // so we have to return the result instead of a `None`
          return res.emit();
        }
        Some(input) => input,
      };

      let head_map = head_map_getter(input.rest());
      let actions = head_map
        .known_map()
        .get(&input.next())
        .unwrap_or(head_map.unknown_fallback());

      match Self::traverse_actions_mut(&mut input, actions, re_lex, &mut re_lexable_factory) {
        // all definition checked, no accepted action
        // but the digested and errors might be updated by the last iteration
        // so we have to return them
        None => return res.emit(),
        Some((output, action_index, muted)) => {
          // update digested, no matter the output is muted or not
          res.digest(output.digested);

          match output.error {
            Some(err) => {
              if muted {
                // err but muted, collect errors and continue
                res
                  .errors()
                  .update((err, create_range(input.start(), output.digested)));
                continue;
              } else {
                // err and not muted, collect error and emit token
                let token = create_token(output.kind, input.start(), output.digested);
                res.errors().update((err, token.range.clone()));
                return res.emit_with_token(
                  token,
                  re_lexable_factory.into_stateless_re_lexable(
                    input.start(),
                    actions.len(),
                    action_index,
                  ),
                );
              }
            }
            None => {
              // else, no error

              // don't emit token if muted
              if muted {
                continue;
              }

              // not muted, emit token
              return res.emit_with_token(
                create_token(output.kind, input.start(), output.digested),
                re_lexable_factory.into_stateless_re_lexable(
                  input.start(),
                  actions.len(),
                  action_index,
                ),
              );
            }
          }
        }
      }
    }
  }

  /// Traverse all actions to find the first accepted action.
  /// Return the output and the index of the accepted action.
  /// If no accepted action, return `None`.
  fn traverse_actions<
    'text,
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

    // prevent unnecessary clone
    if actions.rest().len() == 0 {
      return (None, None);
    }

    let mut state = input.state.clone();
    // TODO: prevent unwrap
    let mut input = ActionInput::new(input.text(), input.start(), &mut state).unwrap();

    (
      traverse_rest(&mut input, actions, re_lex, re_lexable_factory),
      Some(state),
    )
  }

  /// Traverse all actions to find the first accepted action.
  /// Return the output and the index of the accepted action.
  /// If no accepted action, return `None`.
  fn traverse_actions_mut<
    'text,
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::{action::exact, token::MockTokenKind};

  // #[test]
  // fn test_create_token() {
  //   let mut action_state = ();
  //   let input = ActionInput::new("abc", 1, &mut action_state).unwrap();
  //   let output = ActionOutput {
  //     kind: MockTokenKind::new(123),
  //     digested: 1,
  //     error: Some("e"),
  //   };
  //   let token = StatelessLexer::create_token(&input, output);
  //   assert_eq!(token.kind.data, 123);
  //   assert_eq!(token.range.start, 1);
  //   assert_eq!(token.range.end, 2);
  //   // assert_eq!(token.error, Some("e"));
  // }

  #[test]
  fn test_traverse_actions() {
    let mut action_state = ();
    let mut input = ActionInput::new("abc", 1, &mut action_state).unwrap();

    // all actions are checked, no accepted action
    assert!(StatelessLexer::<_>::traverse_actions(
      &mut input,
      &vec![Rc::new(exact("d")),],
      &ReLexContext::default(),
      &mut ()
    )
    .is_none());

    // accept without re-lex context
    assert!(matches!(
      StatelessLexer::<_>::traverse_actions(
        &mut input,
        &vec![
          Rc::new(exact("a")),
          Rc::new(exact("b")),
          Rc::new(exact("c")),
        ],
        &ReLexContext::default(),
        &mut ()
      ),
      Some((
        ActionOutput {
          kind: MockTokenKind { data: () },
          digested: 1,
          error: None
        },
        1
      ))
    ));

    // accept with re-lex context
    assert!(matches!(
      StatelessLexer::<_>::traverse_actions(
        &mut input,
        &vec![
          Rc::new(exact("a")),
          Rc::new(exact("b")),
          Rc::new(exact("c")),
        ],
        &ReLexContext { start: 1, skip: 1 },
        &mut ()
      ),
      Some((
        ActionOutput {
          kind: MockTokenKind { data: () },
          digested: 1,
          error: None
        },
        1
      ))
    ));

    // accepted actions are skipped, no accepted action
    assert!(matches!(
      StatelessLexer::<_>::traverse_actions(
        &mut input,
        &vec![
          Rc::new(exact("a")),
          Rc::new(exact("b")),
          Rc::new(exact("c")),
        ],
        &ReLexContext { start: 1, skip: 2 },
        &mut ()
      ),
      None
    ));

    // ignore re-lex context when start mismatch
    assert!(matches!(
      StatelessLexer::<_>::traverse_actions(
        &mut input,
        &vec![
          Rc::new(exact("a")),
          Rc::new(exact("b")),
          Rc::new(exact("c")),
        ],
        &ReLexContext { start: 0, skip: 3 },
        &mut ()
      ),
      Some((
        ActionOutput {
          kind: MockTokenKind { data: () },
          digested: 1,
          error: None
        },
        1
      ))
    ));

    // TODO: update this
    // // backup action state if fork is enabled and action state is mutated
    // let mut fork = ForkEnabled::default();
    // StatelessLexer::<_>::traverse_actions(
    //   &mut input,
    //   &vec![
    //     Rc::new(exact("a").prepare(|_| {})), // set may_mutate_state to true
    //     Rc::new(exact("b")),
    //     Rc::new(exact("c")),
    //   ],
    //   &ReLexContext { start: 0, skip: 0 },
    //   &mut fork,
    // );
    // let re_lexable: Option<ReLexable<_, ()>> =
    //   fork.into_re_lexable(input.start(), 3, 0, Expectation::default(), 0, "");
    // assert!(re_lexable.is_some());
    // assert!(re_lexable.unwrap().action_state.is_some());

    // // don't backup action state if fork is enabled but action state is not mutated
    // let mut fork = ForkEnabled::default();
    // StatelessLexer::<_>::traverse_actions(
    //   &mut input,
    //   &vec![
    //     Rc::new(exact("a")),
    //     Rc::new(exact("b")),
    //     Rc::new(exact("c")),
    //   ],
    //   &ReLexContext { start: 0, skip: 0 },
    //   &mut fork,
    // );
    // let re_lexable = fork.into_re_lexable(input.start(), 3, 0, Expectation::default(), 0, "");
    // assert!(re_lexable.is_some());
    // assert!(re_lexable.unwrap().action_state.is_none());
  }
}
