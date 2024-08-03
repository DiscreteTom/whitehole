use super::LexerBuilder;
use crate::{
  lexer::{
    action::Action,
    token::{DefaultTokenKindIdBinding, MockTokenKind, TokenKindIdBinding},
  },
  utils::OneOrMore,
};

impl<Kind, State, ErrorType> LexerBuilder<Kind, State, ErrorType> {
  /// Append actions to the builder.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder, token::token_kind};
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
  pub fn append(mut self, actions: impl Into<OneOrMore<Action<Kind, State, ErrorType>>>) -> Self {
    self.actions.extend(actions.into().0);
    self
  }

  /// Append actions with a decorator.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A, B }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::with_error();
  /// // append a single action
  /// builder.append_with(word("A").bind(A), |a| a.error(123));
  /// # let mut builder = LexerBuilder::with_error();
  /// // append multiple actions
  /// builder.append_with([word("A").bind(A), word("B").bind(B)], |a| a.error(123));
  /// # }
  /// ```
  pub fn append_with(
    self,
    actions: impl Into<OneOrMore<Action<Kind, State, ErrorType>>>,
    decorator: impl Fn(Action<Kind, State, ErrorType>) -> Action<Kind, State, ErrorType>,
  ) -> Self {
    self.append(Self::map_actions(actions, decorator))
  }
}

impl<Kind, State, ErrorType> LexerBuilder<TokenKindIdBinding<Kind>, State, ErrorType> {
  /// Append actions and bind them to the default kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, whitespaces, word}, LexerBuilder, token::{token_kind, TokenKindIdBinding}};
  /// # #[token_kind]
  /// # #[derive(Default, Clone)]
  /// # enum MyKind {
  /// #   #[default]
  /// #   Anonymous,
  /// # }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::<TokenKindIdBinding<MyKind>>::new();
  /// // append a single action
  /// builder.append_default(whitespaces());
  /// # let mut builder = LexerBuilder::<TokenKindIdBinding<MyKind>>::new();
  /// // append multiple actions
  /// builder.append_default([whitespaces(), word("_")]);
  /// # }
  /// ```
  pub fn append_default(
    self,
    actions: impl Into<OneOrMore<Action<MockTokenKind<()>, State, ErrorType>>>,
  ) -> Self
  where
    Kind: DefaultTokenKindIdBinding<Kind>,
    State: 'static,
    ErrorType: 'static,
  {
    self.append(Self::map_actions(actions, |a| a.bind_default()))
  }

  /// Append actions with a decorator and bind them to the default kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder, token::{token_kind, TokenKindIdBinding}};
  /// # #[token_kind]
  /// # #[derive(Clone, Default)]
  /// # enum MyKind { #[default] A }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::<TokenKindIdBinding<MyKind>, (), _>::with_error();
  /// // append a single action
  /// builder.append_default_with(word("A"), |a| a.error(123));
  /// # let mut builder = LexerBuilder::<TokenKindIdBinding<MyKind>, (), _>::with_error();
  /// // append multiple actions
  /// builder.append_default_with([word("A"), word("B")], |a| a.error(123));
  /// # }
  /// ```
  pub fn append_default_with(
    self,
    actions: impl Into<OneOrMore<Action<MockTokenKind<()>, State, ErrorType>>>,
    decorator: impl Fn(
      Action<MockTokenKind<()>, State, ErrorType>,
    ) -> Action<MockTokenKind<()>, State, ErrorType>,
  ) -> Self
  where
    Kind: DefaultTokenKindIdBinding<Kind>,
    State: 'static,
    ErrorType: 'static,
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
