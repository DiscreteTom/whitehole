use super::{ActionList, LexerBuilder};
use crate::lexer::{
  action::Action,
  token::{DefaultTokenKindIdBinding, MockTokenKind, TokenKindIdBinding},
};

impl<Kind, ActionState, ErrorType> LexerBuilder<Kind, ActionState, ErrorType> {
  /// Append actions to the builder.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<_>::default();
  /// // append a single action
  /// builder.append(word("A").bind(A));
  /// # let mut builder = LexerBuilder::<_>::default();
  /// // append multiple actions
  /// builder.append([word("A").bind(A), word("B").bind(B)]);
  /// ```
  pub fn append(
    mut self,
    actions: impl Into<ActionList<Action<Kind, ActionState, ErrorType>>>,
  ) -> Self {
    self.actions.extend(actions.into().0);
    self
  }

  /// Append actions with a decorator.
  /// # Examples
  /// The following code won't pass the compile check
  /// because the compiler can't infer the generic parameter type of [`Action`]
  /// when using [`error`](Action::error) to modify the generic parameter type.
  /// ```compile_fail
  /// # use whitehole::lexer::{Action, LexerBuilder, action::exact};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<_, (), i32>::default();
  /// builder.append(exact("A").bind(A).error(123));
  /// ```
  /// The following code will pass the compile.
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<_, (), i32>::default();
  /// // append a single action
  /// builder.append_with(word("A").bind(A), |a| a.error(123));
  /// # let mut builder = LexerBuilder::<_, (), i32>::default();
  /// // append multiple actions
  /// builder.append_with([word("A").bind(A), word("B").bind(B)], |a| a.error(123));
  /// ```
  pub fn append_with(
    self,
    actions: impl Into<ActionList<Action<Kind, ActionState, ErrorType>>>,
    decorator: impl Fn(Action<Kind, ActionState, ErrorType>) -> Action<Kind, ActionState, ErrorType>,
  ) -> Self {
    self.append(Self::map_actions(actions, decorator))
  }
}

impl<Kind, ActionState, ErrorType> LexerBuilder<TokenKindIdBinding<Kind>, ActionState, ErrorType> {
  /// Append actions and bind them to the default kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, whitespaces, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Default, Clone)]
  /// # enum MyKind {
  /// #   #[default]
  /// #   Anonymous,
  /// # }
  /// # let mut builder = LexerBuilder::<_>::default();
  /// // append a single action
  /// builder.append_default(whitespaces());
  /// # let mut builder = LexerBuilder::<_>::default();
  /// // append multiple actions
  /// builder.append_default([whitespaces(), word("_")]);
  /// ```
  pub fn append_default(
    self,
    actions: impl Into<ActionList<Action<MockTokenKind<()>, ActionState, ErrorType>>>,
  ) -> Self
  where
    Kind: DefaultTokenKindIdBinding<Kind>,
    ActionState: 'static,
    ErrorType: 'static,
  {
    self.append(Self::map_actions(actions, |a| a.bind_default()))
  }

  /// Append actions with a decorator and bind them to the default kind.
  /// # Examples
  /// The following code won't pass the compile check
  /// because the compiler can't infer the generic parameter type of [`Action`]
  /// when using [`error`](Action::error) to modify the generic parameter type.
  /// ```compile_fail
  /// # use whitehole::lexer::{Action, LexerBuilder, action::exact};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone, Default)]
  /// # enum MyKind { #[default] A }
  /// # let mut builder = LexerBuilder::<_, (), i32>::default();
  /// builder.append_default(exact("A").error(123));
  /// ```
  /// The following code will pass the compile.
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone, Default)]
  /// # enum MyKind { #[default] A }
  /// # let mut builder = LexerBuilder::<_, (), i32>::default();
  /// // append a single action
  /// builder.append_default_with(word("A"), |a| a.error(123));
  /// # let mut builder = LexerBuilder::<_, (), i32>::default();
  /// // append multiple actions
  /// builder.append_default_with([word("A"), word("B")], |a| a.error(123));
  /// ```
  pub fn append_default_with(
    self,
    actions: impl Into<ActionList<Action<MockTokenKind<()>, ActionState, ErrorType>>>,
    decorator: impl Fn(
      Action<MockTokenKind<()>, ActionState, ErrorType>,
    ) -> Action<MockTokenKind<()>, ActionState, ErrorType>,
  ) -> Self
  where
    Kind: DefaultTokenKindIdBinding<Kind>,
    ActionState: 'static,
    ErrorType: 'static,
  {
    self.append_default(Self::map_actions(actions, decorator))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::{
    action::word,
    token::{SubTokenKind, TokenKindIdBinding},
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
      LexerBuilder::<_>::default()
        .append(word("A").bind(A))
        .actions
        .len(),
      1
    );

    // many
    assert_eq!(
      LexerBuilder::<_>::default()
        .append([word("A").bind(A), word("B").bind(B)])
        .actions
        .len(),
      2
    );
  }

  #[test]
  fn lexer_builder_append_with() {
    // single
    let builder =
      LexerBuilder::<_, (), _>::default().append_with(word("A").bind(A), |a| a.error("123"));
    assert_eq!(builder.actions.len(), 1);
    let stateless = builder.build_stateless();
    assert_eq!(stateless.lex("A").0.token.unwrap().error.unwrap(), "123");

    // many
    let builder = LexerBuilder::<_, (), &str>::default()
      .append_with([word("A").bind(A), word("B").bind(B)], |a| a.error("123"));
    assert_eq!(builder.actions.len(), 2);
    // ensure decorator is applied to all actions
    let stateless = builder.build_stateless();
    assert_eq!(stateless.lex("A").0.token.unwrap().error.unwrap(), "123");
    assert_eq!(stateless.lex("B").0.token.unwrap().error.unwrap(), "123");
  }

  #[test]
  fn lexer_builder_append_default() {
    // single
    let builder = LexerBuilder::<TokenKindIdBinding<MyKind>>::default().append_default(word("A"));
    assert_eq!(builder.actions.len(), 1);
    assert_eq!(builder.actions[0].kind_id(), Anonymous::kind_id());

    // many
    let builder =
      LexerBuilder::<TokenKindIdBinding<MyKind>>::default().append_default([word("A"), word("B")]);
    assert_eq!(builder.actions.len(), 2);
    assert_eq!(builder.actions[0].kind_id(), Anonymous::kind_id());
    assert_eq!(builder.actions[1].kind_id(), Anonymous::kind_id());
  }

  #[test]
  fn lexer_builder_append_default_with() {
    // single
    let builder = LexerBuilder::<TokenKindIdBinding<MyKind>, (), &str>::default()
      .append_default_with(word("A"), |a| a.error("123"));
    assert_eq!(builder.actions.len(), 1);
    assert_eq!(builder.actions[0].kind_id(), Anonymous::kind_id(),);
    let stateless = builder.build_stateless();
    assert_eq!(stateless.lex("A").0.token.unwrap().error.unwrap(), "123");

    // many
    let builder = LexerBuilder::<TokenKindIdBinding<MyKind>, (), &str>::default()
      .append_default_with([word("A"), word("B")], |a| a.error("123"));
    assert_eq!(builder.actions.len(), 2);
    assert_eq!(builder.actions[0].kind_id(), Anonymous::kind_id(),);
    assert_eq!(builder.actions[1].kind_id(), Anonymous::kind_id(),);
    // ensure decorator is applied to all actions
    let stateless = builder.build_stateless();
    assert_eq!(stateless.lex("A").0.token.unwrap().error.unwrap(), "123");
    assert_eq!(stateless.lex("B").0.token.unwrap().error.unwrap(), "123");
  }
}
