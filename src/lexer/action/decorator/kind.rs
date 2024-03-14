use super::AcceptedActionDecoratorContext;
use crate::lexer::{
  action::{ActionInput, ActionInputRestHeadMatcher, ActionOutput, EnhancedActionOutput},
  token::{TokenKind, TokenKindId, TokenKindIdBinding},
  Action,
};
use std::{collections::HashSet, marker::PhantomData};

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
      // TODO: why can't we just store action in MultiKindAction?
      possible_kinds: possible_kinds.into(),
      head_matcher: self.head_matcher,
      maybe_muted: self.maybe_muted,
      may_mutate_state: self.may_mutate_state,
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
        .map(|k| k.id().clone())
        .collect::<HashSet<_>>(),
    )
  }

  // TODO: replace `kinds` with this
  pub fn new_kinds<NewKind, ViaKind>(
    self,
    (possible_kinds, _): (
      HashSet<TokenKindId<TokenKindIdBinding<NewKind>>>,
      PhantomData<ViaKind>,
    ),
  ) -> NewMultiKindAction<NewKind, ViaKind, Kind, ActionState, ErrorType> {
    NewMultiKindAction {
      possible_kinds,
      head_matcher: self.head_matcher,
      maybe_muted: self.maybe_muted,
      may_mutate_state: self.may_mutate_state,
      exec: self.exec,
      via_kind: PhantomData,
    }
  }

  /// Set the kind and the data binding for this action.
  /// Use this if your action can only yield one kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.append(regex(r"^\s+").unwrap().bind(MyKind::A));
  /// ```
  pub fn bind<NewKind>(self, kind: impl Into<NewKind>) -> Action<NewKind, ActionState, ErrorType>
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    NewKind: TokenKind<NewKind> + Clone + 'static,
  {
    let kind = kind.into();
    self
      .kind_ids([kind.id().clone()])
      .select(move |_| kind.clone())
  }
}

pub struct NewMultiKindAction<NewKind, ViaKind, Kind, ActionState, ErrorType> {
  /// See [`Action::possible_kinds`].
  possible_kinds: HashSet<TokenKindId<TokenKindIdBinding<NewKind>>>,
  /// See [`Action::head_matcher`].
  head_matcher: Option<ActionInputRestHeadMatcher>,
  /// See [`Action::maybe_muted`].
  maybe_muted: bool,
  /// See [`Action::may_mutate_state`].
  may_mutate_state: bool,
  /// See [`Action::exec`].
  exec: Box<dyn Fn(&mut ActionInput<ActionState>) -> Option<ActionOutput<Kind, Option<ErrorType>>>>,
  via_kind: PhantomData<ViaKind>,
}

impl<NewKind, ViaKind, Kind, ActionState, ErrorType>
  NewMultiKindAction<NewKind, ViaKind, Kind, ActionState, ErrorType>
{
  pub fn select<F>(self, selector: F) -> Action<TokenKindIdBinding<NewKind>, ActionState, ErrorType>
  where
    ViaKind: Into<TokenKindIdBinding<NewKind>>,
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
      ) -> ViaKind
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
            })
            .into(),
            digested: output.digested,
            muted: output.muted,
            error: output.error,
          }
        })
      }),
      may_mutate_state: self.may_mutate_state,
      maybe_muted: self.maybe_muted,
      possible_kinds: self.possible_kinds,
      head_matcher: self.head_matcher,
    }
  }
}

pub struct MultiKindAction<NewKind, Kind, ActionState, ErrorType> {
  /// See [`Action::possible_kinds`].
  possible_kinds: HashSet<TokenKindId<NewKind>>,
  /// See [`Action::head_matcher`].
  head_matcher: Option<ActionInputRestHeadMatcher>,
  /// See [`Action::maybe_muted`].
  maybe_muted: bool,
  /// See [`Action::may_mutate_state`].
  may_mutate_state: bool,
  /// See [`Action::exec`].
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
      may_mutate_state: self.may_mutate_state,
      maybe_muted: self.maybe_muted,
      possible_kinds: self.possible_kinds,
      head_matcher: self.head_matcher,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{regex, simple};
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

  #[test]
  fn action_bind() {
    let action: Action<Num> = simple(|_| 1).bind(Even);
    assert_eq!(action.possible_kinds.len(), 1);
    assert!(action.possible_kinds.contains(&Even.id()));
    assert!(matches!(
      action.exec(&mut ActionInput::new("A", 0, &mut ())),
      Some(ActionOutput {
        kind: Even,
        digested: 1,
        muted: false,
        error: None
      })
    ));
  }
}
