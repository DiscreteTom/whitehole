use super::LexerBuilder;
use crate::{
  kind::{DefaultSubKind, MockKind},
  lexer::action::Action,
  utils::OneOrMore,
};

impl<'a, Kind, State, Heap> LexerBuilder<'a, Kind, State, Heap> {
  /// Define [`muted`](Action::muted) actions by calling [`Action::mute`].
  /// # Examples
  /// ```
  /// # use whitehole::{kind::whitehole_kind, lexer::{action::{Action, word}, builder::LexerBuilder}};
  /// # #[whitehole_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A, B }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// // append a single action
  /// builder.ignore(word("A").bind(A));
  /// # let mut builder = LexerBuilder::new();
  /// // append multiple actions
  /// builder.ignore([word("A").bind(A), word("B").bind(B)]);
  /// # }
  /// ```
  #[inline]
  pub fn ignore(self, actions: impl Into<OneOrMore<Action<'a, Kind, State, Heap>>>) -> Self {
    self.append(Self::map_actions(actions, |a| a.mute()))
  }

  /// Define [`muted`](Action::muted) actions by calling [`Action::mute`] with a decorator.
  /// # Examples
  /// ```
  /// # use whitehole::{kind::whitehole_kind, lexer::{action::{Action, word}, builder::LexerBuilder}};
  /// # #[whitehole_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A, B }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// // append a single action
  /// builder.ignore_with(word("A").bind(A), |a| a.reject());
  /// # let mut builder = LexerBuilder::new();
  /// // append multiple actions
  /// builder.ignore_with([word("A").bind(A), word("B").bind(B)], |a| a.reject());
  /// # }
  /// ```
  #[inline]
  pub fn ignore_with(
    self,
    actions: impl Into<OneOrMore<Action<'a, Kind, State, Heap>>>,
    decorator: impl Fn(Action<Kind, State, Heap>) -> Action<Kind, State, Heap> + 'a,
  ) -> Self {
    self.ignore(Self::map_actions(actions, decorator))
  }

  /// Define [`muted`](Action::muted) actions by calling [`Action::mute`] and bind them to the default kind.
  /// # Examples
  /// ```
  /// # use whitehole::{kind::whitehole_kind, lexer::{action::{Action, whitespaces, word}, builder::LexerBuilder}};
  /// # #[whitehole_kind]
  /// # #[derive(Default, Clone)]
  /// # enum MyKind {
  /// #   #[default]
  /// #   Anonymous,
  /// # }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::<MyKind>::new();
  /// // append a single action
  /// builder.ignore_default(whitespaces());
  /// # let mut builder = LexerBuilder::<MyKind>::new();
  /// // append multiple actions
  /// builder.ignore_default([whitespaces(), word("_")]);
  /// # }
  /// ```
  #[inline]
  pub fn ignore_default(
    self,
    actions: impl Into<OneOrMore<Action<'a, MockKind<()>, State, Heap>>>,
  ) -> Self
  where
    Kind: DefaultSubKind + Default,
    State: 'a,
    Heap: 'a,
  {
    self.ignore(Self::map_actions(actions, |a| a.bind_default()))
  }

  /// Define [`muted`](Action::muted) actions by calling [`Action::mute`] with a decorator and bind them to the default kind.
  /// # Examples
  /// ```
  /// # use whitehole::{kind::whitehole_kind, lexer::{action::{Action, word}, builder::LexerBuilder}};
  /// # #[whitehole_kind]
  /// # #[derive(Clone, Default)]
  /// # enum MyKind { #[default] A }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::<MyKind>::new();
  /// // append a single action
  /// builder.ignore_default_with(word("A"), |a| a.reject());
  /// # let mut builder = LexerBuilder::<MyKind>::new();
  /// // append multiple actions
  /// builder.ignore_default_with([word("A"), word("B")], |a| a.reject());
  /// # }
  /// ```
  #[inline]
  pub fn ignore_default_with(
    self,
    actions: impl Into<OneOrMore<Action<'a, MockKind<()>, State, Heap>>>,
    decorator: impl Fn(Action<MockKind<()>, State, Heap>) -> Action<MockKind<()>, State, Heap> + 'a,
  ) -> Self
  where
    Kind: DefaultSubKind + Default,
    State: 'a,
    Heap: 'a,
  {
    self.ignore_default(Self::map_actions(actions, decorator))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{kind::SubKind, lexer::action::word};
  use whitehole_macros::_whitehole_kind;

  #[_whitehole_kind]
  #[derive(Default, Clone, Debug)]
  enum MyKind {
    #[default]
    Anonymous,
    A,
    B,
  }

  #[test]
  fn lexer_builder_ignore() {
    // single
    let builder = LexerBuilder::new().ignore(word("A").bind(A));
    assert_eq!(builder.actions.len(), 1);
    assert!(builder.actions[0].muted());

    // many
    let builder = LexerBuilder::new().ignore([word("A").bind(A), word("B").bind(B)]);
    assert_eq!(builder.actions.len(), 2);
    assert!(builder.actions[0].muted());
    assert!(builder.actions[1].muted());
  }

  #[test]
  fn lexer_builder_ignore_with() {
    // single
    let builder =
      LexerBuilder::new().ignore_with(word("A").bind(A), |a| a.unchecked_head_unknown());
    assert_eq!(builder.actions.len(), 1);
    assert!(builder.actions[0].muted());
    assert!(builder.actions[0].head().is_some());

    // many
    let builder = LexerBuilder::new().ignore_with([word("A").bind(A), word("B").bind(B)], |a| {
      a.unchecked_head_unknown()
    });
    assert_eq!(builder.actions.len(), 2);
    assert!(builder.actions[0].muted());
    assert!(builder.actions[1].muted());
    assert!(builder.actions[0].head().is_some());
    assert!(builder.actions[1].head().is_some());
  }

  #[test]
  fn lexer_builder_ignore_default() {
    // single
    let builder = LexerBuilder::new().ignore_default(word("A"));
    assert_eq!(builder.actions.len(), 1);
    assert_eq!(builder.actions[0].kind(), Anonymous::kind_id());
    assert!(builder.actions[0].muted());

    // many
    let builder = LexerBuilder::new().ignore_default([word("A"), word("B")]);
    assert_eq!(builder.actions.len(), 2);
    assert_eq!(builder.actions[0].kind(), Anonymous::kind_id());
    assert_eq!(builder.actions[1].kind(), Anonymous::kind_id());
    assert!(builder.actions[0].muted());
    assert!(builder.actions[1].muted());
  }

  #[test]
  fn lexer_builder_ignore_default_with() {
    // single
    let builder =
      LexerBuilder::new().ignore_default_with(word("A"), |a| a.unchecked_head_unknown());
    assert_eq!(builder.actions.len(), 1);
    assert_eq!(builder.actions[0].kind(), Anonymous::kind_id());
    assert!(builder.actions[0].muted());
    assert!(builder.actions[0].head().is_some());

    // many
    let builder = LexerBuilder::new()
      .ignore_default_with([word("A"), word("B")], |a| a.unchecked_head_unknown());
    assert_eq!(builder.actions.len(), 2);
    assert_eq!(builder.actions[0].kind(), Anonymous::kind_id());
    assert_eq!(builder.actions[1].kind(), Anonymous::kind_id());
    assert!(builder.actions[0].muted());
    assert!(builder.actions[1].muted());
    assert!(builder.actions[0].head().is_some());
    assert!(builder.actions[1].head().is_some());
  }
}
