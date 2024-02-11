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
  /// This is used to accelerate the lexing process when lexing with expected kinds.
  pub fn kinds<NewKind: 'static>(
    self,
    possible_kinds: impl Into<Vec<NewKind>>,
  ) -> MultiKindAction<NewKind, Kind, ActionState, ErrorType>
  where
    NewKind: TokenKind<NewKind>,
  {
    MultiKindAction {
      possible_kinds: possible_kinds.into().iter().map(|kind| kind.id()).collect(),
      head_matcher: self.head_matcher,
      maybe_muted: self.maybe_muted,
      exec: self.exec,
    }
  }

  /// Set [`Action::possible_kinds`].
  /// This is used to accelerate the lexing process when lexing with expected kinds.
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
