use super::LexerBuilder;
use crate::{
  lexer::{
    action::Action,
    token::{DefaultTokenKindIdBinding, MockTokenKind},
  },
  utils::OneOrMore,
};

impl<Kind, State, ErrorType> LexerBuilder<Kind, State, ErrorType> {
  /// Define [`muted`](Action::muted) actions by calling [`Action::mute`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
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
  pub fn ignore(self, actions: impl Into<OneOrMore<Action<Kind, State, ErrorType>>>) -> Self
  where
    Kind: 'static,
    State: 'static,
    ErrorType: 'static,
  {
    self.append(Self::map_actions(actions, |a| a.mute()))
  }

  /// Define [`muted`](Action::muted) actions by calling [`Action::mute`] with a decorator.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
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
  pub fn ignore_with<F>(
    self,
    actions: impl Into<OneOrMore<Action<Kind, State, ErrorType>>>,
    decorator: F,
  ) -> Self
  where
    Kind: 'static,
    State: 'static,
    ErrorType: 'static,
    F: Fn(Action<Kind, State, ErrorType>) -> Action<Kind, State, ErrorType>,
  {
    self.ignore(Self::map_actions(actions, decorator))
  }

  /// Define [`muted`](Action::muted) actions by calling [`Action::mute`] and bind them to the default kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, whitespaces, word}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
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
  pub fn ignore_default(
    self,
    actions: impl Into<OneOrMore<Action<MockTokenKind<()>, State, ErrorType>>>,
  ) -> Self
  where
    Kind: DefaultTokenKindIdBinding<Kind> + 'static,
    State: 'static,
    ErrorType: 'static,
  {
    self.ignore(Self::map_actions(actions, |a| a.bind_default()))
  }

  /// Define [`muted`](Action::muted) actions by calling [`Action::mute`] with a decorator and bind them to the default kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
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
  pub fn ignore_default_with<F>(
    self,
    actions: impl Into<OneOrMore<Action<MockTokenKind<()>, State, ErrorType>>>,
    decorator: F,
  ) -> Self
  where
    Kind: DefaultTokenKindIdBinding<Kind> + 'static,
    State: 'static,
    ErrorType: 'static,
    F: Fn(
      Action<MockTokenKind<()>, State, ErrorType>,
    ) -> Action<MockTokenKind<()>, State, ErrorType>,
  {
    self.ignore_default(Self::map_actions(actions, decorator))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::{action::word, token::SubTokenKind};
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
