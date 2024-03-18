use crate::lexer::{action::ActionInputRestHeadMatcher, Action};
use std::{collections::HashSet, ops::RangeInclusive};

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  /// Set [`Action::head_matcher`] to [`OneOf`](ActionInputRestHeadMatcher::OneOf).
  /// The provided parameter will NOT be checked, you have to make sure it's logically correct.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define(A, regex(r"^A").unwrap().unchecked_head_in(['A']));
  /// ```
  pub fn unchecked_head_in(mut self, char_set: impl Into<HashSet<char>>) -> Self {
    self.head_matcher = Some(ActionInputRestHeadMatcher::OneOf(char_set.into()));
    self
  }

  /// Set [`Action::head_matcher`] to [`OneOf`](ActionInputRestHeadMatcher::OneOf)
  /// with the given range.
  /// The provided parameter will NOT be checked, you have to make sure it's logically correct.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define(A, regex(r"^[A-Z]").unwrap().unchecked_head_in_range('A'..='Z'));
  pub fn unchecked_head_in_range(
    self,
    range: impl Into<RangeInclusive<char>>,
  ) -> Action<Kind, ActionState, ErrorType> {
    self.unchecked_head_in(range.into().into_iter().collect::<HashSet<_>>())
  }

  /// Set [`Action::head_matcher`] to [`Not`](ActionInputRestHeadMatcher::Not).
  /// The provided parameter will NOT be checked, you have to make sure it's logically correct.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define(MyKind::A, regex(r"^[^A]").unwrap().unchecked_head_not(['A']));
  /// ```
  pub fn unchecked_head_not(mut self, char_set: impl Into<HashSet<char>>) -> Self {
    self.head_matcher = Some(ActionInputRestHeadMatcher::Not(char_set.into()));
    self
  }

  /// Set [`Action::head_matcher`] to [`Unknown`](ActionInputRestHeadMatcher::Unknown).
  /// The provided parameter will NOT be checked, you have to make sure it's logically correct.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define(MyKind::A, regex(r"^.").unwrap().unchecked_head_unknown());
  /// ```
  pub fn unchecked_head_unknown(mut self) -> Self {
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
    let action: Action<_> = simple(|_| 1).unchecked_head_in(['a']);
    assert!(matches!(
      action.head_matcher,
      Some(ActionInputRestHeadMatcher::OneOf(set)) if set == HashSet::from(['a'])
    ));
  }

  #[test]
  fn action_head_in_range() {
    let action: Action<_> = simple(|_| 1).unchecked_head_in_range('a'..='z');
    assert!(matches!(
      action.head_matcher,
      Some(ActionInputRestHeadMatcher::OneOf(set)) if set == ('a'..='z').into_iter().collect::<HashSet<_>>()
    ));
  }

  #[test]
  fn action_head_not() {
    let action: Action<_> = regex(r"^a").unwrap().unchecked_head_not(['b']);
    assert!(matches!(
      action.head_matcher,
      Some(ActionInputRestHeadMatcher::Not(set)) if set == HashSet::from(['b'])
    ));
  }

  #[test]
  fn action_head_unknown() {
    let action: Action<_> = simple(|_| 1).unchecked_head_unknown();
    assert!(matches!(
      action.head_matcher,
      Some(ActionInputRestHeadMatcher::Unknown)
    ));
  }
}
