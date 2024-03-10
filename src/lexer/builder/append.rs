use super::{ActionList, LexerBuilder};
use crate::lexer::{token::TokenKind, Action};

impl<Kind, ActionState, ErrorType> LexerBuilder<Kind, ActionState, ErrorType> {
  /// Append actions to the builder.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append a single action
  /// builder.append(word("A").bind(A));
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append multiple actions
  /// builder.append([word("A").bind(A), word("B").bind(B)]);
  /// ```
  pub fn append(
    mut self,
    actions: impl Into<ActionList<Action<Kind, ActionState, ErrorType>>>,
  ) -> Self {
    actions
      .into()
      .0
      .into_iter()
      .for_each(|action| self.actions.push(action));
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
  /// # let mut builder = LexerBuilder::<MyKind, (), i32>::default();
  /// builder.append(exact("A").bind(A).error(123));
  /// ```
  /// The following code will pass the compile.
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<MyKind, (), i32>::default();
  /// // append a single action
  /// builder.append_with(word("A").bind(A), |a| a.error(123));
  /// # let mut builder = LexerBuilder::<MyKind, (), i32>::default();
  /// // append multiple actions
  /// builder.append_with([word("A").bind(A), word("B").bind(B)], |a| a.error(123));
  /// ```
  pub fn append_with<F>(
    self,
    actions: impl Into<ActionList<Action<Kind, ActionState, ErrorType>>>,
    decorator: F,
  ) -> Self
  where
    Kind: 'static,
    F: Fn(Action<Kind, ActionState, ErrorType>) -> Action<Kind, ActionState, ErrorType>,
  {
    self.append(Self::map_actions(actions, decorator))
  }

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
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append a single action
  /// builder.append_default(whitespaces());
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append multiple actions
  /// builder.append_default([whitespaces(), word("_")]);
  /// ```
  pub fn append_default(
    self,
    actions: impl Into<ActionList<Action<(), ActionState, ErrorType>>>,
  ) -> Self
  where
    Kind: TokenKind<Kind> + Default + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    self.append(Self::map_actions(actions, |a| a.bind(Kind::default())))
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
  /// # let mut builder = LexerBuilder::<MyKind, (), i32>::default();
  /// builder.append_default(exact("A").error(123));
  /// ```
  /// The following code will pass the compile.
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone, Default)]
  /// # enum MyKind { #[default] A }
  /// # let mut builder = LexerBuilder::<MyKind, (), i32>::default();
  /// // append a single action
  /// builder.append_default_with(word("A"), |a| a.error(123));
  /// # let mut builder = LexerBuilder::<MyKind, (), i32>::default();
  /// // append multiple actions
  /// builder.append_default_with([word("A"), word("B")], |a| a.error(123));
  /// ```
  pub fn append_default_with<F>(
    self,
    actions: impl Into<ActionList<Action<(), ActionState, ErrorType>>>,
    decorator: F,
  ) -> Self
  where
    Kind: TokenKind<Kind> + Default + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(Action<(), ActionState, ErrorType>) -> Action<(), ActionState, ErrorType>,
  {
    self.append_default(Self::map_actions(actions, decorator))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::word;
  use whitehole_macros::_TokenKind;
  use MyKind::*;

  #[derive(_TokenKind, Default, Clone)]
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
      LexerBuilder::<MyKind>::default()
        .append(word("A").bind(A))
        .build_stateless()
        .actions()
        .len(),
      1
    );

    // many
    assert_eq!(
      LexerBuilder::<MyKind>::default()
        .append([word("A").bind(A), word("B").bind(B)])
        .build_stateless()
        .actions()
        .len(),
      2
    );
  }

  #[test]
  fn lexer_builder_append_with() {
    // single
    let stateless = LexerBuilder::<MyKind, (), &str>::default()
      .append_with(word("A").bind(A), |a| a.error("123"))
      .build_stateless();
    assert_eq!(stateless.actions().len(), 1);
    assert_eq!(stateless.lex("A").0.token.unwrap().error.unwrap(), "123");

    // many
    let stateless = LexerBuilder::<MyKind, (), &str>::default()
      .append_with([word("A").bind(A), word("B").bind(B)], |a| a.error("123"))
      .build_stateless();
    assert_eq!(stateless.actions().len(), 2);
    // ensure decorator is applied to all actions
    assert_eq!(stateless.lex("A").0.token.unwrap().error.unwrap(), "123");
    assert_eq!(stateless.lex("B").0.token.unwrap().error.unwrap(), "123");
  }

  #[test]
  fn lexer_builder_append_default() {
    // single
    let stateless = LexerBuilder::<MyKind>::default()
      .append_default(word("A"))
      .build_stateless();
    assert_eq!(stateless.actions().len(), 1);
    assert_eq!(stateless.actions()[0].possible_kinds().len(), 1);
    assert!(stateless.actions()[0]
      .possible_kinds()
      .contains(&Anonymous.id()),);

    // many
    let stateless = LexerBuilder::<MyKind>::default()
      .append_default([word("A"), word("B")])
      .build_stateless();
    assert_eq!(stateless.actions().len(), 2);
    assert_eq!(stateless.actions()[0].possible_kinds().len(), 1);
    assert!(stateless.actions()[0]
      .possible_kinds()
      .contains(&Anonymous.id()),);
    assert_eq!(stateless.actions()[1].possible_kinds().len(), 1);
    assert!(stateless.actions()[1]
      .possible_kinds()
      .contains(&Anonymous.id()));
  }

  #[test]
  fn lexer_builder_append_default_with() {
    // single
    let stateless = LexerBuilder::<MyKind, (), &str>::default()
      .append_default_with(word("A"), |a| a.error("123"))
      .build_stateless();
    assert_eq!(stateless.actions().len(), 1);
    assert_eq!(stateless.actions()[0].possible_kinds().len(), 1);
    assert!(stateless.actions()[0]
      .possible_kinds()
      .contains(&Anonymous.id()),);
    assert_eq!(stateless.lex("A").0.token.unwrap().error.unwrap(), "123");

    // many
    let stateless = LexerBuilder::<MyKind, (), &str>::default()
      .append_default_with([word("A"), word("B")], |a| a.error("123"))
      .build_stateless();
    assert_eq!(stateless.actions().len(), 2);
    assert_eq!(stateless.actions()[0].possible_kinds().len(), 1);
    assert!(stateless.actions()[0]
      .possible_kinds()
      .contains(&Anonymous.id()),);
    assert_eq!(stateless.actions()[1].possible_kinds().len(), 1);
    assert!(stateless.actions()[1]
      .possible_kinds()
      .contains(&Anonymous.id()),);
    // ensure decorator is applied to all actions
    assert_eq!(stateless.lex("A").0.token.unwrap().error.unwrap(), "123");
    assert_eq!(stateless.lex("B").0.token.unwrap().error.unwrap(), "123");
  }
}
