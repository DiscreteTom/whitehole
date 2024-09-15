use super::LexerBuilder;
use crate::{
  lexer::{
    action::Action,
    token::{DefaultTokenKind, MockTokenKind},
  },
  utils::OneOrMore,
};

impl<'a, Kind, State, Heap> LexerBuilder<'a, Kind, State, Heap> {
  /// Append actions to the builder.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, builder::LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A, B }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// // append a single action
  /// builder.append(word("A").bind(A));
  /// # let mut builder = LexerBuilder::new();
  /// // append multiple actions
  /// builder.append([word("A").bind(A), word("B").bind(B)]);
  /// # }
  /// ```
  #[inline]
  pub fn append(mut self, actions: impl Into<OneOrMore<Action<'a, Kind, State, Heap>>>) -> Self {
    self.actions.extend(actions.into().0);
    self
  }

  /// Append actions with a decorator.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, builder::LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A, B }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// // append a single action
  /// builder.append_with(word("A").bind(A), |a| a.reject());
  /// # let mut builder = LexerBuilder::new();
  /// // append multiple actions
  /// builder.append_with([word("A").bind(A), word("B").bind(B)], |a| a.reject());
  /// # }
  /// ```
  #[inline]
  pub fn append_with(
    self,
    actions: impl Into<OneOrMore<Action<'a, Kind, State, Heap>>>,
    decorator: impl Fn(Action<Kind, State, Heap>) -> Action<Kind, State, Heap> + 'a,
  ) -> Self {
    self.append(Self::map_actions(actions, decorator))
  }

  /// Append actions and bind them to the default kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, whitespaces, word}, builder::LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Default, Clone)]
  /// # enum MyKind {
  /// #   #[default]
  /// #   Anonymous,
  /// # }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::<MyKind>::new();
  /// // append a single action
  /// builder.append_default(whitespaces());
  /// # let mut builder = LexerBuilder::<MyKind>::new();
  /// // append multiple actions
  /// builder.append_default([whitespaces(), word("_")]);
  /// # }
  /// ```
  #[inline]
  pub fn append_default(
    self,
    actions: impl Into<OneOrMore<Action<'a, MockTokenKind<()>, State, Heap>>>,
  ) -> Self
  where
    Kind: DefaultTokenKind + Default,
    State: 'a,
    Heap: 'a,
  {
    self.append(Self::map_actions(actions, |a| a.bind_default()))
  }

  /// Append actions with a decorator and bind them to the default kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, builder::LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone, Default)]
  /// # enum MyKind { #[default] A }
  /// # fn main() {
  /// # let mut builder: LexerBuilder<MyKind> = LexerBuilder::new();
  /// // append a single action
  /// builder.append_default_with(word("A"), |a| a.reject());
  /// # let mut builder: LexerBuilder<MyKind> = LexerBuilder::new();
  /// // append multiple actions
  /// builder.append_default_with([word("A"), word("B")], |a| a.reject());
  /// # }
  /// ```
  #[inline]
  pub fn append_default_with(
    self,
    actions: impl Into<OneOrMore<Action<'a, MockTokenKind<()>, State, Heap>>>,
    decorator: impl Fn(Action<MockTokenKind<()>, State, Heap>) -> Action<MockTokenKind<()>, State, Heap>
      + 'a,
  ) -> Self
  where
    Kind: DefaultTokenKind + Default,
    State: 'a,
    Heap: 'a,
  {
    self.append_default(Self::map_actions(actions, decorator))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::{
    action::{exact, word},
    token::SubTokenKind,
  };
  use whitehole_macros::_token_kind;

  #[_token_kind]
  #[derive(Default, Clone, Debug)]
  enum MyKind {
    #[default]
    Anonymous,
    A,
    B,
  }

  #[test]
  fn lexer_builder_append() {
    // single
    assert_eq!(
      LexerBuilder::new().append(word("A").bind(A)).actions.len(),
      1
    );

    // many
    assert_eq!(
      LexerBuilder::new()
        .append([word("A").bind(A), word("B").bind(B)])
        .actions
        .len(),
      2
    );
  }

  #[test]
  fn lexer_builder_append_with() {
    // single
    let builder = LexerBuilder::new().append_with(word("A").bind(A), |a| a.mute());
    assert_eq!(builder.actions.len(), 1);
    assert!(builder.actions[0].muted());

    // many
    let builder =
      LexerBuilder::new().append_with([exact("A").bind(A), exact("B").bind(B)], |a| a.mute());
    assert_eq!(builder.actions.len(), 2);
    // ensure decorator is applied to all actions
    assert!(builder.actions[0].muted());
    assert!(builder.actions[1].muted());
  }

  #[test]
  fn lexer_builder_append_default() {
    // single
    let builder = LexerBuilder::new().append_default(word("A"));
    assert_eq!(builder.actions.len(), 1);
    assert_eq!(builder.actions[0].kind(), Anonymous::kind_id());

    // many
    let builder = LexerBuilder::new().append_default([word("A"), word("B")]);
    assert_eq!(builder.actions.len(), 2);
    assert_eq!(builder.actions[0].kind(), Anonymous::kind_id());
    assert_eq!(builder.actions[1].kind(), Anonymous::kind_id());
  }

  #[test]
  fn lexer_builder_append_default_with() {
    // single
    let builder = LexerBuilder::new().append_default_with(word("A"), |a| a.mute());
    assert_eq!(builder.actions.len(), 1);
    assert_eq!(builder.actions[0].kind(), Anonymous::kind_id(),);
    assert!(builder.actions[0].muted());

    // many
    let builder = LexerBuilder::new().append_default_with([exact("A"), exact("B")], |a| a.mute());
    assert_eq!(builder.actions.len(), 2);
    assert_eq!(builder.actions[0].kind(), Anonymous::kind_id(),);
    assert_eq!(builder.actions[1].kind(), Anonymous::kind_id(),);
    // ensure decorator is applied to all actions
    assert!(builder.actions[0].muted());
    assert!(builder.actions[1].muted());
  }
}
