use super::{output::StatelessOutput, HeadMap, StatelessLexer};
use crate::{
  lexer::{
    action::{Action, ActionInput, ActionOutput},
    re_lex::{ReLexContext, ReLexableFactory},
    token::{Range, Token, TokenKindIdProvider},
  },
  utils::Accumulator,
};
use std::rc::Rc;

impl<Kind, ActionState, ErrorType> StatelessLexer<Kind, ActionState, ErrorType> {
  pub(super) fn execute_actions<
    'text,
    'expect_literal,
    'head_map,
    ErrAcc: Accumulator<(ErrorType, Range)>,
    ReLexableFactoryType: ReLexableFactory<'text, Kind, ActionState, ErrorType>,
    StatelessOutputType: StatelessOutput<Token<Kind>, ErrAcc, ReLexableFactoryType::StatelessReLexableType>,
  >(
    head_map_getter: impl Fn(
      &ActionInput<ActionState>,
    ) -> &'head_map HeadMap<Kind, ActionState, ErrorType>,
    re_lex: &ReLexContext,
    text: &'text str,
    start: usize,
    state: &mut ActionState,
    mut re_lexable_factory: ReLexableFactoryType,
    mut res: StatelessOutputType,
  ) -> StatelessOutputType
  where
    Kind: TokenKindIdProvider<Kind> + 'static,
    ActionState: 'head_map,
    ErrorType: 'head_map,
  {
    loop {
      // all actions will reuse this action input in this iteration
      let mut input = match ActionInput::new(text, start + res.digested(), state) {
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
        .get(&input.next())
        .unwrap_or(head_map.unknown_fallback());

      match Self::traverse_actions(&mut input, actions, re_lex, &mut re_lexable_factory) {
        // all definition checked, no accepted action
        // but the digested and errors might be updated by the last iteration
        // so we have to return them
        None => return res,
        Some((output, action_index)) => {
          // update digested, no matter the output is muted or not
          res.digest(output.digested);

          match output.error {
            Some(err) => {
              if actions[action_index].muted() {
                // err but muted, collect errors and continue
                res
                  .err_acc_mut()
                  .update((err, Self::construct_range(&input, output.digested)));
                continue;
              } else {
                // err and not muted, collect error and emit token
                let token = Self::create_token(&input, output.kind, output.digested);
                res.err_acc_mut().update((err, token.range.clone()));
                res.emit(
                  token,
                  re_lexable_factory.into_stateless_re_lexable(
                    input.start(),
                    actions.len(),
                    action_index,
                  ),
                );

                return res;
              }
            }
            None => {
              // else, no error

              // don't emit token if muted
              if actions[action_index].muted() {
                continue;
              }

              // not muted, emit token
              res.emit(
                Self::create_token(&input, output.kind, output.digested),
                re_lexable_factory.into_stateless_re_lexable(
                  input.start(),
                  actions.len(),
                  action_index,
                ),
              );

              return res;
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
    'expect_literal,
    ReLexableFactoryType: ReLexableFactory<'text, Kind, ActionState, ErrorType>,
  >(
    input: &mut ActionInput<ActionState>,
    actions: &[Rc<Action<Kind, ActionState, ErrorType>>],
    re_lex: &ReLexContext,
    re_lexable_factory: &mut ReLexableFactoryType,
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
      // TODO: when fork is disabled, skip this check to optimize performance?
      // TODO: pre-calculate whether a whole head_map won't mutate state.
      if action.may_mutate_state() {
        re_lexable_factory.before_mutate_action_state(input.state);
      }

      if let Some(output) = action.exec(input) {
        return Some((output, i));
      }
    }

    // all actions are checked, no accepted action
    None
  }

  fn create_token(input: &ActionInput<ActionState>, kind: Kind, digested: usize) -> Token<Kind> {
    Token {
      range: Self::construct_range(input, digested),
      kind,
    }
  }

  #[inline]
  fn construct_range(input: &ActionInput<ActionState>, digested: usize) -> Range {
    Range {
      start: input.start(),
      end: input.start() + digested,
    }
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
