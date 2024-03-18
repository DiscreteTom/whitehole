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

use super::token::TokenKindId;
use std::collections::HashSet;

/// See [`Action::head_matcher`].
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
  /// See [`Self::kind_id`].
  kind_id: TokenKindId<Kind>,
  /// See [`Self::head_matcher`].
  head_matcher: Option<ActionInputRestHeadMatcher>,
  /// See [`Self::maybe_muted`].
  maybe_muted: bool,
  /// See [`Self::may_mutate_state`].
  may_mutate_state: bool,
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
  /// Every action should have this field set by [`Self::bind`] and [`Self::select`].
  pub fn kind_id(&self) -> &TokenKindId<Kind> {
    &self.kind_id
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
  fn action_getters_default() {
    let action: Action<()> = Action {
      exec: Box::new(|_| None),
      kind_id: TokenKindId::new(0),
      head_matcher: None,
      maybe_muted: false,
      may_mutate_state: false,
    };
    assert!(!action.maybe_muted());
    assert!(action.never_muted());
    assert!(!action.may_mutate_state());
    assert!(action.never_mutate_state());
    assert_eq!(action.kind_id().0, 0);
    assert!(action.head_matcher().is_none());
  }

  #[test]
  fn action_getters() {
    let action: Action<()> = Action {
      exec: Box::new(|_| None),
      kind_id: TokenKindId::new(1),
      head_matcher: Some(ActionInputRestHeadMatcher::OneOf(HashSet::from(['a']))),
      maybe_muted: true,
      may_mutate_state: true,
    };
    assert!(action.maybe_muted());
    assert!(!action.never_muted());
    assert!(action.may_mutate_state());
    assert!(!action.never_mutate_state());
    assert_eq!(action.kind_id().0, 1);
    assert!(
      matches!(action.head_matcher(), Some(ActionInputRestHeadMatcher::OneOf(set)) if set == &HashSet::from(['a']))
    );
  }
}
