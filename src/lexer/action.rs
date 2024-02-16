pub mod builder;
pub mod decorator;
pub mod input;
pub mod output;
pub mod regex;
pub mod select;
pub mod simple;
pub mod utils;

use self::{
  input::ActionInput,
  output::{ActionOutput, ActionOutputWithoutKind},
};
use super::token::TokenKindId;
use std::collections::HashSet;

pub enum ActionInputRestHeadMatcher {
  OneOf(HashSet<char>),
  Not(HashSet<char>),
  /// Match any characters that are not in known `OneOf` or `Not`.
  Unknown,
}

pub struct Action<Kind: 'static, ActionState: 'static = (), ErrorType: 'static = ()> {
  /// This flag is to indicate whether this action's output might be muted.
  /// The lexer will based on this flag to accelerate the lexing process.
  /// If `true`, this action's output may be muted.
  /// If `false`, this action's output will never be muted.
  /// For most cases this field will be set automatically,
  /// so don't set this field unless you know what you are doing.
  pub maybe_muted: bool,

  /// This is used to accelerate expectational lexing.
  /// Every action should have this field set by [`Action::kinds`].
  possible_kinds: HashSet<TokenKindId<Kind>>,
  /// This is used to accelerate lexing by the first character
  /// of the rest of the input. This is optional.
  head_matcher: Option<ActionInputRestHeadMatcher>,
  exec: Box<dyn Fn(&mut ActionInput<ActionState>) -> Option<ActionOutput<Kind, ErrorType>>>,
}

impl<ActionState, ErrorType> Action<(), ActionState, ErrorType> {
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
  /// Shortcut for `!self.maybe_muted`.
  pub fn never_muted(&self) -> bool {
    !self.maybe_muted
  }

  /// Should only be set by [`Action::kinds`].
  pub fn possible_kinds(&self) -> &HashSet<TokenKindId<Kind>> {
    &self.possible_kinds
  }

  /// Should only be set by [`Action::head_in`] or [`Action::head_not`].
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
