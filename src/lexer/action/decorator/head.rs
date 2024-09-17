use crate::lexer::action::{echo_with, Action, HeadMatcher};
use std::{collections::HashSet, ops::RangeInclusive};

impl<Kind, State, Heap> Action<'_, Kind, State, Heap> {
  /// Set [`Action::head`] to [`HeadMatcher::OneOf`].
  /// # Caveats
  /// The provided parameter will NOT be checked, you have to make sure it's logically correct.
  /// # Examples
  /// ```
  /// # use whitehole::{kind::whitehole_kind, lexer::{action::{Action, regex}, builder::LexerBuilder}};
  /// # #[whitehole_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// builder.define(A, regex(r"^A").unchecked_head_in(['A']));
  /// # }
  /// ```
  #[inline]
  pub fn unchecked_head_in(mut self, char_set: impl Into<HashSet<char>>) -> Self {
    echo_with!(self, self.head = Some(HeadMatcher::OneOf(char_set.into())))
  }

  /// Set [`Action::head`] to [`HeadMatcher::OneOf`]
  /// with the given range.
  /// # Caveats
  /// The provided parameter will NOT be checked, you have to make sure it's logically correct.
  /// # Examples
  /// ```
  /// # use whitehole::{kind::whitehole_kind, lexer::{action::{Action, regex}, builder::LexerBuilder}};
  /// # #[whitehole_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// builder.define(A, regex(r"^[A-Z]").unchecked_head_in_range('A'..='Z'));
  /// # }
  #[inline]
  pub fn unchecked_head_in_range(self, range: impl Into<RangeInclusive<char>>) -> Self {
    self.unchecked_head_in(range.into().collect::<HashSet<_>>())
  }

  /// Set [`Action::head`] to [`HeadMatcher::Not`].
  /// # Caveats
  /// The provided parameter will NOT be checked, you have to make sure it's logically correct.
  /// # Examples
  /// ```
  /// # use whitehole::{kind::whitehole_kind, lexer::{action::{Action, regex}, builder::LexerBuilder}};
  /// # #[whitehole_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// builder.define(A, regex(r"^[^A]").unchecked_head_not(['A']));
  /// # }
  /// ```
  #[inline]
  pub fn unchecked_head_not(mut self, char_set: impl Into<HashSet<char>>) -> Self {
    echo_with!(self, self.head = Some(HeadMatcher::Not(char_set.into())))
  }

  /// Set [`Action::head`] to [`HeadMatcher::Unknown`].
  /// # Caveats
  /// The provided parameter will NOT be checked, you have to make sure it's logically correct.
  /// # Examples
  /// ```
  /// # use whitehole::{kind::whitehole_kind, lexer::{action::{Action, regex}, builder::LexerBuilder}};
  /// # #[whitehole_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// builder.define(A, regex(r"^.").unchecked_head_unknown());
  /// # }
  /// ```
  #[inline]
  pub fn unchecked_head_unknown(mut self) -> Self {
    echo_with!(self, self.head = Some(HeadMatcher::Unknown))
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
      Some(HeadMatcher::OneOf(set)) if set == ('a'..='z').collect::<HashSet<_>>()
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
