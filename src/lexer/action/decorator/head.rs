use crate::lexer::{action::ActionInputRestHeadMatcher, Action};
use std::{collections::HashSet, ops::RangeInclusive};

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  /// Set [`Action::head_matcher`] to [`OneOf`](ActionInputRestHeadMatcher::OneOf).
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define(A, regex(r"^A").unwrap().head_in(['A']));
  /// ```
  pub fn head_in(mut self, char_set: impl Into<HashSet<char>>) -> Self {
    self.head_matcher = Some(ActionInputRestHeadMatcher::OneOf(char_set.into()));
    self
  }

  /// Set [`Action::head_matcher`] to [`OneOf`](ActionInputRestHeadMatcher::OneOf)
  /// with the given range.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define(A, regex(r"^[A-Z]").unwrap().head_in_range('A'..='Z'));
  pub fn head_in_range(
    self,
    range: impl Into<RangeInclusive<char>>,
  ) -> Action<Kind, ActionState, ErrorType> {
    self.head_in(range.into().into_iter().collect::<HashSet<_>>())
  }

  /// Set [`Action::head_matcher`] to [`Not`](ActionInputRestHeadMatcher::Not).
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define(MyKind::A, regex(r"^[^A]").unwrap().head_not(['A']));
  /// ```
  pub fn head_not(mut self, char_set: impl Into<HashSet<char>>) -> Self {
    self.head_matcher = Some(ActionInputRestHeadMatcher::Not(char_set.into()));
    self
  }

  /// Set [`Action::head_matcher`] to [`Unknown`](ActionInputRestHeadMatcher::Unknown).
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define(MyKind::A, regex(r"^.").unwrap().head_unknown());
  /// ```
  pub fn head_unknown(mut self) -> Self {
    self.head_matcher = Some(ActionInputRestHeadMatcher::Unknown);
    self
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{regex, simple};

  #[test]
  fn action_head_in() {
    let action: Action<()> = simple(|_| 1).head_in(['a']);
    assert!(matches!(
      action.head_matcher,
      Some(ActionInputRestHeadMatcher::OneOf(set)) if set.contains(&'a') && set.len() == 1
    ));
  }

  #[test]
  fn action_head_in_range() {
    let action: Action<()> = simple(|_| 1).head_in_range('a'..='z');
    assert!(matches!(
      action.head_matcher,
      Some(ActionInputRestHeadMatcher::OneOf(set)) if set.contains(&'a') && set.contains(&'z') && set.len() == 26
    ));
  }

  #[test]
  fn action_head_not() {
    let action: Action<(), (), ()> = regex(r"^a").unwrap().head_not(['b']);
    assert!(matches!(
      action.head_matcher,
      Some(ActionInputRestHeadMatcher::Not(set)) if set.contains(&'b') && set.len() == 1
    ));
  }

  #[test]
  fn action_head_unknown() {
    let action: Action<()> = simple(|_| 1).head_unknown();
    assert!(matches!(
      action.head_matcher,
      Some(ActionInputRestHeadMatcher::Unknown)
    ));
  }
}
