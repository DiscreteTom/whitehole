mod decorator;
mod input;
mod output;
mod regex;
mod simple;
mod utils;

pub use decorator::*;
pub use input::*;
pub use output::*;
pub use regex::*;
pub use simple::*;
pub use utils::*;

use super::token::{MockTokenKind, TokenKind, TokenKindId};
use std::collections::HashSet;

pub enum ActionInputRestHeadMatcher {
  OneOf(HashSet<char>),
  Not(HashSet<char>),
  /// Match any characters that are not known in
  /// [`OneOf`](ActionInputRestHeadMatcher::OneOf) or [`Not`](ActionInputRestHeadMatcher::Not).
  Unknown,
}

pub struct Action<Kind, ActionState = (), ErrorType = ()> {
  // input is mutable so the action can mutate the action state.
  exec: Box<dyn Fn(&mut ActionInput<ActionState>) -> Option<ActionOutput<Kind, Option<ErrorType>>>>,
  /// See [`Self::possible_kinds`].
  possible_kinds: HashSet<TokenKindId<Kind>>,
  /// See [`Self::head_matcher`].
  head_matcher: Option<ActionInputRestHeadMatcher>,
  /// See [`Self::maybe_muted`].
  maybe_muted: bool,
  /// See [`Self::may_mutate_state`].
  may_mutate_state: bool,
}

impl<ActionState, ErrorType> Action<(), ActionState, ErrorType> {
  /// Create a new action with no kind.
  /// To set the kind, use [`Self::bind`], [`Self::kinds`] or [`Self::kind_ids`].
  // TODO: make this private or unsafe
  pub fn new<F>(exec: F) -> Self
  where
    F: Fn(&mut ActionInput<ActionState>) -> Option<ActionOutputWithoutKind<Option<ErrorType>>>
      + 'static,
  {
    Action {
      exec: Box::new(move |input| {
        // transform ActionOutputWithoutKind tp ActionOutput
        exec(input).map(|output| output.into())
      }),
      possible_kinds: HashSet::new(),
      head_matcher: None,
      maybe_muted: false,
      may_mutate_state: false,
    }
  }
}

impl<ActionState, ErrorType, T> Action<MockTokenKind<T>, ActionState, ErrorType> {
  /// Create an action with the [`MockTokenKind`] as the kind.
  /// This is usually used to pass data to downstream actions.
  // TODO: update comments
  pub fn with_data<F>(exec: F) -> Self
  where
    F: Fn(&mut ActionInput<ActionState>) -> Option<ActionOutput<MockTokenKind<T>, Option<ErrorType>>>
      + 'static,
  {
    Action {
      exec: Box::new(exec),
      possible_kinds: MockTokenKind::possible_kinds(),
      head_matcher: None,
      maybe_muted: false,
      may_mutate_state: false,
    }
    // since there is just on possible kinds in MockTokenKind
    // we don't need to call `action.kinds().select()` here
  }
}

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  /// This flag is to indicate whether this action's output might be muted.
  /// The lexer will based on this flag to accelerate the lexing process.
  /// If `true`, this action's output may be muted.
  /// If `false`, this action's output will never be muted.
  /// This field should only be set via [`Self::mute`] or [`Self::mute_if`].
  pub fn maybe_muted(&self) -> bool {
    self.maybe_muted
  }

  /// Equals to `!self.maybe_muted()`.
  /// See [`Self::maybe_muted`].
  pub fn never_muted(&self) -> bool {
    !self.maybe_muted
  }

  /// This flag is to indicate whether this action might mutate the `ActionState`.
  /// This is used to lazy-clone the `ActionState` when lexing with fork enabled.
  /// This will be `false` by default, and will be `true` if [`Self::prepare`]
  /// or [`Self::callback`] is called.
  pub fn may_mutate_state(&self) -> bool {
    self.may_mutate_state
  }

  /// Equals to `!self.may_mutate_state()`.
  /// See [`Self::may_mutate_state`].
  pub fn never_mutate_state(&self) -> bool {
    !self.may_mutate_state
  }

  /// This is used to accelerate expectational lexing.
  /// Every action should have this field set by [`Self::bind`] [`Self::kinds`] or [`Self::kind_ids`].
  pub fn possible_kinds(&self) -> &HashSet<TokenKindId<Kind>> {
    &self.possible_kinds
  }

  /// This is used to accelerate lexing by the first character
  /// of the rest of the input. This is optional but highly recommended.
  /// This should only be set by [`Self::head_in`], [`Self::head_in_range`],
  /// [`Self::head_not`] and [`Self::head_unknown`].
  pub fn head_matcher(&self) -> &Option<ActionInputRestHeadMatcher> {
    &self.head_matcher
  }

  pub fn exec(
    &self,
    input: &mut ActionInput<ActionState>,
  ) -> Option<ActionOutput<Kind, Option<ErrorType>>> {
    (self.exec)(input)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn action_new() {
    let action: Action<()> = Action::new(|_| None);
    assert!(!action.maybe_muted);
    assert_eq!(action.possible_kinds().len(), 0);
    assert!(action.head_matcher().is_none());
    assert!(action.never_muted())
  }

  #[test]
  fn action_with_data() {
    let action: Action<MockTokenKind<()>> = Action::with_data(|_| None);
    assert!(!action.maybe_muted);
    assert_eq!(action.possible_kinds().len(), 1);
    assert!(action.possible_kinds().contains(&TokenKindId::new(0)));
    assert!(action.head_matcher().is_none());
    assert!(action.never_muted())
  }
}
