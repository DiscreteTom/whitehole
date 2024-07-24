use crate::lexer::action::{Action, HeadMatcher};
use std::{collections::HashSet, ops::RangeInclusive};

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  /// Set [`Action::head`] to [`OneOf`](HeadMatcher::OneOf).
  /// The provided parameter will NOT be checked, you have to make sure it's logically correct.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// builder.define(A, regex(r"^A").unchecked_head_in(['A']));
  /// # }
  /// ```
  pub fn unchecked_head_in(mut self, char_set: impl Into<HashSet<char>>) -> Self {
    self.head = Some(HeadMatcher::OneOf(char_set.into()));
    self
  }

  /// Set [`Action::head`] to [`OneOf`](HeadMatcher::OneOf)
  /// with the given range.
  /// The provided parameter will NOT be checked, you have to make sure it's logically correct.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// builder.define(A, regex(r"^[A-Z]").unchecked_head_in_range('A'..='Z'));
  /// # }
  pub fn unchecked_head_in_range(self, range: impl Into<RangeInclusive<char>>) -> Self {
    self.unchecked_head_in(range.into().into_iter().collect::<HashSet<_>>())
  }

  /// Set [`Action::head`] to [`Not`](HeadMatcher::Not).
  /// The provided parameter will NOT be checked, you have to make sure it's logically correct.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// builder.define(A, regex(r"^[^A]").unchecked_head_not(['A']));
  /// # }
  /// ```
  pub fn unchecked_head_not(mut self, char_set: impl Into<HashSet<char>>) -> Self {
    self.head = Some(HeadMatcher::Not(char_set.into()));
    self
  }

  /// Set [`Action::head`] to [`Unknown`](HeadMatcher::Unknown).
  /// The provided parameter will NOT be checked, you have to make sure it's logically correct.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// builder.define(A, regex(r"^.").unchecked_head_unknown());
  /// # }
  /// ```
  pub fn unchecked_head_unknown(mut self) -> Self {
    self.head = Some(HeadMatcher::Unknown);
    self
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{regex, simple};

  #[test]
  fn action_head_in() {
    let action: Action<_> = Action::from(simple(|_| 1)).unchecked_head_in(['a']);
    assert!(matches!(
      action.head,
      Some(HeadMatcher::OneOf(set)) if set == HashSet::from(['a'])
    ));
  }

  #[test]
  fn action_head_in_range() {
    let action: Action<_> = Action::from(simple(|_| 1)).unchecked_head_in_range('a'..='z');
    assert!(matches!(
      action.head,
      Some(HeadMatcher::OneOf(set)) if set == ('a'..='z').into_iter().collect::<HashSet<_>>()
    ));
  }

  #[test]
  fn action_head_not() {
    let action: Action<_> = Action::from(regex(r"^a")).unchecked_head_not(['b']);
    assert!(matches!(
      action.head,
      Some(HeadMatcher::Not(set)) if set == HashSet::from(['b'])
    ));
  }

  #[test]
  fn action_head_unknown() {
    let action: Action<_> = Action::from(simple(|_| 1)).unchecked_head_unknown();
    assert!(matches!(action.head, Some(HeadMatcher::Unknown)));
  }
}
