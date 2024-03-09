use crate::lexer::{
  action::{ActionInput, ActionInputRestHeadMatcher, ActionOutput, EnhancedActionOutput},
  token::{TokenKind, TokenKindId},
  Action,
};
use std::collections::HashSet;

use super::AcceptedActionDecoratorContext;

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  /// Set [`Action::possible_kinds`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::regex;
  /// # use whitehole::lexer::action::Action;
  /// # use whitehole::lexer::token::TokenKind;
  /// # use whitehole_macros::TokenKind;
  /// # use Num::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum Num { Even, Odd }
  /// # let action: Action<Num, (), ()> =
  /// regex(r"^\d+").unwrap().kind_ids([Even.id(), Odd.id()]).select(|ctx| {
  ///   if ctx.output.content().parse::<u32>().unwrap() % 2 == 0 {
  ///     Even
  ///   } else {
  ///     Odd
  ///   }
  /// });
  /// ```
  pub fn kind_ids<NewKind>(
    self,
    possible_kinds: impl Into<HashSet<TokenKindId<NewKind>>>,
  ) -> MultiKindAction<NewKind, Kind, ActionState, ErrorType> {
    MultiKindAction {
      possible_kinds: possible_kinds.into(),
      head_matcher: self.head_matcher,
      maybe_muted: self.maybe_muted,
      exec: self.exec,
    }
  }

  /// Set [`Action::possible_kinds`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::regex;
  /// # use whitehole::lexer::action::Action;
  /// # use whitehole_macros::TokenKind;
  /// # use Num::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum Num { Even, Odd }
  /// # let action: Action<Num, (), ()> =
  /// regex(r"^\d+").unwrap().kinds([Even, Odd]).select(|ctx| {
  ///   if ctx.output.content().parse::<u32>().unwrap() % 2 == 0 {
  ///     Even
  ///   } else {
  ///     Odd
  ///   }
  /// });
  /// ```
  pub fn kinds<NewKind>(
    self,
    possible_kinds: impl Into<Vec<NewKind>>,
  ) -> MultiKindAction<NewKind, Kind, ActionState, ErrorType>
  where
    NewKind: TokenKind<NewKind>,
  {
    self.kind_ids(
      possible_kinds
        .into()
        .into_iter()
        .map(TokenKindId::from)
        .collect::<HashSet<_>>(),
    )
  }
}

pub struct MultiKindAction<NewKind, Kind, ActionState, ErrorType> {
  possible_kinds: HashSet<TokenKindId<NewKind>>,
  head_matcher: Option<ActionInputRestHeadMatcher>,
  maybe_muted: bool,
  exec: Box<dyn Fn(&mut ActionInput<ActionState>) -> Option<ActionOutput<Kind, Option<ErrorType>>>>,
}

impl<NewKind, Kind, ActionState, ErrorType> MultiKindAction<NewKind, Kind, ActionState, ErrorType> {
  /// Define a selector to select a kind from action's kinds by action's input and output.
  /// **Be ware**: the result won't be checked against `possible_kinds`
  /// so make sure the result is in `possible_kinds`.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::regex;
  /// # use whitehole::lexer::action::Action;
  /// # use whitehole_macros::TokenKind;
  /// # use Num::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum Num { Even, Odd }
  /// # let action: Action<Num, (), ()> =
  /// regex(r"^\d+").unwrap().kinds([Even, Odd]).select(|ctx| {
  ///   if ctx.output.content().parse::<u32>().unwrap() % 2 == 0 {
  ///     Even
  ///   } else {
  ///     Odd
  ///   }
  /// });
  /// ```
  /// You can consume the original output's kind to get the data.
  /// ```
  /// # use whitehole::lexer::{action::simple_option_with_data, Action};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind)]
  /// # enum MyKind { Numbers(Vec<i32>) }
  /// # let action: Action<MyKind, (), ()> =
  /// simple_option_with_data(|_| Some((1, vec![1])))
  ///     .kinds([Numbers(vec![])])
  ///     .select(|ctx| Numbers(ctx.output.base.kind.data));
  /// ```
  // TODO: better example
  pub fn select<F>(self, selector: F) -> Action<NewKind, ActionState, ErrorType>
  where
    NewKind: 'static,
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(
        AcceptedActionDecoratorContext<
          // user can't mutate the input
          &ActionInput<ActionState>,
          // output is consumed except the error
          EnhancedActionOutput<Kind, &Option<ErrorType>>,
        >,
      ) -> NewKind
      + 'static,
  {
    let exec = self.exec;
    Action {
      exec: Box::new(move |input| {
        exec(input).map(|output| {
          ActionOutput {
            kind: selector(AcceptedActionDecoratorContext {
              output: EnhancedActionOutput::new(
                input,
                // construct a new ActionOutput
                ActionOutput {
                  // consume the original output.kind
                  kind: output.kind,
                  digested: output.digested,
                  muted: output.muted,
                  // but don't consume the error
                  error: &output.error,
                },
              ),
              input,
            }),
            digested: output.digested,
            muted: output.muted,
            error: output.error,
          }
        })
      }),
      maybe_muted: self.maybe_muted,
      possible_kinds: self.possible_kinds,
      head_matcher: self.head_matcher,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::regex;
  use whitehole_macros::_TokenKind;
  use Num::*;

  #[derive(_TokenKind, Clone)]
  enum Num {
    Even,
    Odd,
  }

  fn test_runner<F>(f: F)
  where
    F: Fn(Action<()>) -> Action<Num>,
  {
    let action = f(regex(r"^\d+").unwrap().head_unknown().mute(true));

    // ensure the possible kinds is set
    assert_eq!(action.possible_kinds.len(), 2);
    assert!(action.possible_kinds.contains(&Even.id()));
    assert!(action.possible_kinds.contains(&Odd.id()));

    // ensure other fields are not changed
    assert!(action.maybe_muted);
    assert!(matches!(
      action.head_matcher,
      Some(ActionInputRestHeadMatcher::Unknown)
    ));

    // ensure the result is correct
    assert!(matches!(
      action.exec(&mut ActionInput::new("123", 0, &mut ())),
      Some(ActionOutput {
        kind: Odd,
        digested: 3,
        muted: true,
        error: None
      })
    ));
    assert!(matches!(
      action.exec(&mut ActionInput::new("124", 0, &mut ())),
      Some(ActionOutput {
        kind: Even,
        digested: 3,
        muted: true,
        error: None
      })
    ));
  }

  #[test]
  fn action_kind_ids() {
    test_runner(|a| {
      a.kind_ids([Even.id(), Odd.id()]).select(|ctx| {
        if ctx.output.content().parse::<u32>().unwrap() % 2 == 0 {
          Even
        } else {
          Odd
        }
      })
    });
  }

  #[test]
  fn action_kinds() {
    test_runner(|a| {
      a.kinds([Even, Odd]).select(|ctx| {
        if ctx.output.content().parse::<u32>().unwrap() % 2 == 0 {
          Even
        } else {
          Odd
        }
      })
    });
  }
}
