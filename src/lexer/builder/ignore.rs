use super::{ActionList, LexerBuilder};
use crate::lexer::{token::TokenKind, Action};

impl<Kind, ActionState, ErrorType> LexerBuilder<Kind, ActionState, ErrorType> {
  /// Define [`muted`](Action::maybe_muted) actions by calling [`Action::mute`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append a single action
  /// builder.ignore(word("A").bind(A));
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append multiple actions
  /// builder.ignore([word("A").bind(A), word("B").bind(B)]);
  /// ```
  pub fn ignore(self, actions: impl Into<ActionList<Action<Kind, ActionState, ErrorType>>>) -> Self
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    self.append(Self::map_actions(actions, |a| a.mute(true)))
  }

  /// Define [`muted`](Action::maybe_muted) actions by calling [`Action::mute`] with a decorator.
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
  /// builder.ignore(exact("A").bind(A).error(123));
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
  /// builder.ignore_with(word("A").bind(A), |a| a.error(123));
  /// # let mut builder = LexerBuilder::<MyKind, (), i32>::default();
  /// // append multiple actions
  /// builder.ignore_with([word("A").bind(A), word("B").bind(B)], |a| a.error(123));
  /// ```
  pub fn ignore_with<F>(
    self,
    actions: impl Into<ActionList<Action<Kind, ActionState, ErrorType>>>,
    decorator: F,
  ) -> Self
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(Action<Kind, ActionState, ErrorType>) -> Action<Kind, ActionState, ErrorType>,
  {
    self.ignore(Self::map_actions(actions, decorator))
  }

  /// Define [`muted`](Action::maybe_muted) actions by calling [`Action::mute`] and bind them to the default kind.
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
  /// builder.ignore_default(whitespaces());
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append multiple actions
  /// builder.ignore_default([whitespaces(), word("_")]);
  /// ```
  pub fn ignore_default(
    self,
    actions: impl Into<ActionList<Action<(), ActionState, ErrorType>>>,
  ) -> Self
  where
    Kind: TokenKind<Kind> + Default + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    self.ignore(Self::map_actions(actions, |a| a.bind(Kind::default())))
  }

  /// Define [`muted`](Action::maybe_muted) actions by calling [`Action::mute`] with a decorator and bind them to the default kind.
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
  /// builder.ignore_default(exact("A").error(123));
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
  /// builder.ignore_default_with(word("A"), |a| a.error(123));
  /// # let mut builder = LexerBuilder::<MyKind, (), i32>::default();
  /// // append multiple actions
  /// builder.ignore_default_with([word("A"), word("B")], |a| a.error(123));
  /// ```
  pub fn ignore_default_with<F>(
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
    self.ignore_default(Self::map_actions(actions, decorator))
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
  fn lexer_builder_ignore() {
    // single
    let stateless = LexerBuilder::<MyKind>::default()
      .ignore(word("A").bind(A))
      .build_stateless();
    assert_eq!(stateless.actions().len(), 1);
    assert!(stateless.actions()[0].maybe_muted());

    // many
    let stateless = LexerBuilder::<MyKind>::default()
      .ignore([word("A").bind(A), word("B").bind(B)])
      .build_stateless();
    assert_eq!(stateless.actions().len(), 2);
    assert!(stateless.actions()[0].maybe_muted());
    assert!(stateless.actions()[1].maybe_muted());
  }

  #[test]
  fn lexer_builder_ignore_with() {
    // single
    let stateless = LexerBuilder::<MyKind, (), i32>::default()
      .ignore_with(word("A").bind(A), |a| a.error(123))
      .build_stateless();
    assert_eq!(stateless.actions().len(), 1);
    assert!(stateless.actions()[0].maybe_muted());

    // many
    let stateless = LexerBuilder::<MyKind, (), i32>::default()
      .ignore_with([word("A").bind(A), word("B").bind(B)], |a| a.error(123))
      .build_stateless();
    assert_eq!(stateless.actions().len(), 2);
    assert!(stateless.actions()[0].maybe_muted());
    assert!(stateless.actions()[1].maybe_muted());
  }

  #[test]
  fn lexer_builder_ignore_default() {
    // single
    let stateless = LexerBuilder::<MyKind>::default()
      .ignore_default(word("A"))
      .build_stateless();
    assert_eq!(stateless.actions().len(), 1);
    assert_eq!(stateless.actions()[0].possible_kinds().len(), 1);
    assert!(stateless.actions()[0]
      .possible_kinds()
      .contains(&Anonymous.id()),);
    assert!(stateless.actions()[0].maybe_muted());

    // many
    let stateless = LexerBuilder::<MyKind>::default()
      .ignore_default([word("A"), word("B")])
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
    assert!(stateless.actions()[0].maybe_muted());
    assert!(stateless.actions()[1].maybe_muted());
  }

  #[test]
  fn lexer_builder_ignore_default_with() {
    // single
    let stateless = LexerBuilder::<MyKind, (), i32>::default()
      .ignore_default_with(word("A"), |a| a.error(123))
      .build_stateless();
    assert_eq!(stateless.actions().len(), 1);
    assert_eq!(stateless.actions()[0].possible_kinds().len(), 1);
    assert!(stateless.actions()[0]
      .possible_kinds()
      .contains(&Anonymous.id()),);
    assert!(stateless.actions()[0].maybe_muted());

    // many
    let stateless = LexerBuilder::<MyKind, (), i32>::default()
      .ignore_default_with([word("A"), word("B")], |a| a.error(123))
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
    assert!(stateless.actions()[0].maybe_muted());
    assert!(stateless.actions()[1].maybe_muted());
  }
}
