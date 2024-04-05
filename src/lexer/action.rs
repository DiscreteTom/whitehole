//! ## Design
//!
//! For a better engineering experience, the lexer is designed to be modular
//! and consists of many [`Action`]s. Each action is a small piece of logic
//! which will digest some bytes from the rest of the text input, and yield a token or not.
//! By doing so, users can easily compose their own lexer by combining existing actions,
//! or create their own actions by modifying existing ones.
//! Users can also share their actions with others by publishing them as a library,
//! or build higher-level libraries to generate actions.
//!
//! Besides, [`Action`]s may be considered heavy because the `Action` and
//! [`ActionOutput`] has many fields and when we modify an `Action`
//! with [`decorator`]s they may be destructed/created many times.
//! To solve this problem, we can use [`SubAction`]
//! which is a light weight version of [`Action`]
//! that only returns how many bytes are digested.
//! For most cases, users should build the `Action`'s logic with `SubAction`s,
//! then transform the `SubAction` into an `Action`, then finish
//! the `Action` with [`decorator`]s.
//!
//! ## For Developers
//!
//! Here is the recommended order of reading the source code:
//!
//! 1. [`self::input`]
//! 2. [`self::output`]
//! 3. [`self`]
//! 4. [`self::sub_action`]
//! 5. [`self::decorator`]
//! 6. [`self::simple`]
//! 7. [`self::regex`]
//! 8. [`self::utils`]

mod decorator;
mod input;
mod output;
mod regex;
mod simple;
mod sub_action;
mod utils;

pub use decorator::*;
pub use input::*;
pub use output::*;
pub use regex::*;
pub use simple::*;
pub use sub_action::*;
pub use utils::*;

use super::token::TokenKindId;
use std::collections::HashSet;

/// See [`Action::head_matcher`].
pub enum HeadMatcher {
  OneOf(HashSet<char>),
  Not(HashSet<char>),
  /// Match any characters that are not known in
  /// [`OneOf`](HeadMatcher::OneOf) or [`Not`](HeadMatcher::Not).
  Unknown,
}

/// To create this, use [`simple`], [`regex`], [`utils`] or [`SubAction::into`](SubAction).
pub struct Action<Kind: 'static, ActionState = (), ErrorType = ()> {
  // input is mutable so the action can mutate the action state.
  exec: Box<dyn Fn(&mut ActionInput<ActionState>) -> Option<ActionOutput<Kind, Option<ErrorType>>>>,
  /// See [`Self::kind_id`].
  kind_id: &'static TokenKindId<Kind>,
  /// See [`Self::head_matcher`].
  head_matcher: Option<HeadMatcher>,
  /// See [`Self::muted`].
  muted: bool,
  /// See [`Self::may_mutate_state`].
  may_mutate_state: bool,
}

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  /// This is used to accelerate expectational lexing.
  /// Every action must have this field set by [`Self::bind`],
  /// [`Self::bind_default`] or [`Self::select`].
  pub fn kind_id(&self) -> &TokenKindId<Kind> {
    &self.kind_id
  }

  /// This is used to accelerate lexing by the first character
  /// of the rest of the input. This is optional but highly recommended.
  /// Some [`utils`] already set this field safely and you should use them as much as possible.
  /// If you want to set this field manually,
  /// this could be set by [`Self::unchecked_head_in`], [`Self::unchecked_head_in_range`],
  /// [`Self::unchecked_head_not`] or [`Self::unchecked_head_unknown`].
  pub fn head_matcher(&self) -> &Option<HeadMatcher> {
    &self.head_matcher
  }

  /// This flag is to indicate whether this action's output is muted.
  /// The lexer will based on this flag to accelerate the lexing process.
  /// This field could be set via [`Self::mute`] or [`Self::unmute`].
  pub fn muted(&self) -> bool {
    self.muted
  }

  /// This flag is to indicate whether this action might mutate the `ActionState`.
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
    static KIND_ID: TokenKindId<()> = TokenKindId::new(1);
    let action: Action<()> = Action {
      exec: Box::new(|_| None),
      kind_id: &KIND_ID,
      head_matcher: None,
      muted: false,
      may_mutate_state: false,
    };
    assert!(!action.muted());
    assert!(!action.may_mutate_state());
    assert!(action.never_mutate_state());
    assert_eq!(action.kind_id(), &TokenKindId::new(0));
    assert!(action.head_matcher().is_none());
  }

  #[test]
  fn action_getters() {
    static KIND_ID: TokenKindId<()> = TokenKindId::new(1);
    let action: Action<()> = Action {
      exec: Box::new(|_| None),
      kind_id: &KIND_ID,
      head_matcher: Some(HeadMatcher::OneOf(HashSet::from(['a']))),
      muted: true,
      may_mutate_state: true,
    };
    assert!(action.muted());
    assert!(action.may_mutate_state());
    assert!(!action.never_mutate_state());
    assert_eq!(action.kind_id(), &TokenKindId::new(1));
    assert!(
      matches!(action.head_matcher(), Some(HeadMatcher::OneOf(set)) if set == &HashSet::from(['a']))
    );
  }
}
