pub mod builder;
pub mod decorator;
pub mod input;
pub mod output;
mod regex;
pub mod select;
mod simple;
mod utils;

pub use regex::regex;
pub use simple::simple;
pub use utils::*;

use self::{
  input::ActionInput,
  output::{ActionOutput, ActionOutputWithoutKind},
};
use super::token::TokenKindId;
use std::collections::HashSet;

pub enum ActionInputRestHeadMatcher {
  OneOf(HashSet<char>),
  Not(HashSet<char>),
  /// Match any characters that are not known in `OneOf` or `Not`.
  Unknown,
}

pub struct Action<Kind, ActionState = (), ErrorType = ()> {
  /// This flag is to indicate whether this action's output might be muted.
  /// The lexer will based on this flag to accelerate the lexing process.
  /// If `true`, this action's output may be muted.
  /// If `false`, this action's output will never be muted.
  /// For most cases this field will be set automatically (e.g. via [`Action::mute`] or [`Action::mute_if`]),
  /// so don't set this field manually unless you know what you are doing.
  pub maybe_muted: bool,

  /// See [`Action::possible_kinds`].
  possible_kinds: HashSet<TokenKindId<Kind>>,
  /// See [`Action::head_matcher`].
  head_matcher: Option<ActionInputRestHeadMatcher>,
  // input is mutable so the action can mutate the action state.
  exec: Box<dyn Fn(&mut ActionInput<ActionState>) -> Option<ActionOutput<Kind, ErrorType>>>,
}

impl<ActionState, ErrorType> Action<(), ActionState, ErrorType> {
  /// Create a new action with no kind.
  /// To set the kind, use [`Action::bind`], [`Action::kinds`] or [`Action::kind_ids`].
  pub fn new<F>(exec: F) -> Self
  where
    F: Fn(&mut ActionInput<ActionState>) -> Option<ActionOutputWithoutKind<ErrorType>> + 'static,
  {
    Action {
      maybe_muted: false,
      possible_kinds: HashSet::new(),
      head_matcher: None,
      exec: Box::new(move |input| exec(input).map(|output| output.into())),
    }
  }
}

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  /// Equals to `!self.maybe_muted`.
  pub fn never_muted(&self) -> bool {
    !self.maybe_muted
  }

  /// This is used to accelerate expectational lexing.
  /// Every action should have this field set by [`Action::bind`] [`Action::kinds`] or [`Action::kind_ids`].
  pub fn possible_kinds(&self) -> &HashSet<TokenKindId<Kind>> {
    &self.possible_kinds
  }

  /// This is used to accelerate lexing by the first character
  /// of the rest of the input. This is optional but highly recommended.
  /// Should only be set by [`Action::head_in`], [`Action::head_not`] and [`Action::head_unknown`].
  pub fn head_matcher(&self) -> &Option<ActionInputRestHeadMatcher> {
    &self.head_matcher
  }

  pub fn exec(
    &self,
    input: &mut ActionInput<ActionState>,
  ) -> Option<ActionOutput<Kind, ErrorType>> {
    (self.exec)(input)
  }
}
