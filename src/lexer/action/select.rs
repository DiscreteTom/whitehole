use super::{
  decorator::AcceptedActionDecoratorContext,
  input::ActionInput,
  output::{ActionOutput, EnhancedActionOutput},
  Action, ActionInputRestHeadMatcher,
};
use crate::lexer::token::{TokenKind, TokenKindId};
use std::collections::HashSet;

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
  pub fn kind_ids<NewKind: 'static>(
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
  pub fn kinds<NewKind: 'static>(
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
  exec: Box<dyn Fn(&mut ActionInput<ActionState>) -> Option<ActionOutput<Kind, ErrorType>>>,
}

impl<NewKind, Kind: 'static, ActionState: 'static, ErrorType: 'static>
  MultiKindAction<NewKind, Kind, ActionState, ErrorType>
{
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
  pub fn select<F>(self, selector: F) -> Action<NewKind, ActionState, ErrorType>
  where
    F: Fn(&AcceptedActionDecoratorContext<Kind, ActionState, ErrorType>) -> NewKind + 'static,
  {
    let exec = self.exec;
    Action {
      exec: Box::new(move |input: &mut ActionInput<ActionState>| {
        exec(input).map(|output| {
          let ctx = AcceptedActionDecoratorContext {
            output: EnhancedActionOutput::new(input, output),
            input,
          };
          ActionOutput {
            kind: selector(&ctx),
            digested: ctx.output.raw.digested,
            muted: ctx.output.raw.muted,
            error: ctx.output.raw.error,
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
    let mut state = ();
    let mut input = ActionInput::new("123", 0, &mut state);
    assert!(matches!(
      action.exec(&mut input),
      Some(ActionOutput {
        kind: Odd,
        digested: 3,
        muted: true,
        error: None
      })
    ));
    let mut input = ActionInput::new("124", 0, &mut state);
    assert!(matches!(
      action.exec(&mut input),
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
