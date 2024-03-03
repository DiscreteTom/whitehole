use super::{ActionList, LexerBuilder};
use crate::lexer::{action::ActionBuilder, token::TokenKind, Action};

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

  /// Append actions with [`ActionBuilder`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append a single action
  /// builder.append_with(|a| a.from(word("A")).bind(A).into());
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append multiple actions
  /// builder.append_with(|a| [
  ///   a.from(word("A")).bind(A),
  ///   a.from(word("B")).bind(B)
  /// ].into());
  /// ```
  pub fn append_with<F>(self, factory: F) -> Self
  where
    F: FnOnce(
      ActionBuilder<ActionState, ErrorType>,
    ) -> ActionList<Action<Kind, ActionState, ErrorType>>,
  {
    self.append(factory(ActionBuilder::default()))
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

  /// Append actions with [`ActionBuilder`] and bind it to the default kind.
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
  /// builder.append_default_with(|a| a.from(whitespaces()).into());
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append multiple actions
  /// builder.append_default_with(|a| [
  ///   a.from(whitespaces()),
  ///   a.from(word("_"))
  /// ].into());
  /// ```
  pub fn append_default_with<F>(self, factory: F) -> Self
  where
    Kind: TokenKind<Kind> + Default + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: FnOnce(
      ActionBuilder<ActionState, ErrorType>,
    ) -> ActionList<Action<(), ActionState, ErrorType>>,
  {
    self.append_default(factory(ActionBuilder::default()))
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
    assert_eq!(
      LexerBuilder::<MyKind>::default()
        .append_with(|a| a.from(word("A")).bind(A).into())
        .build_stateless()
        .actions()
        .len(),
      1
    );

    // many
    assert_eq!(
      LexerBuilder::<MyKind>::default()
        .append_with(|a| [a.from(word("A")).bind(A), a.from(word("B")).bind(B)].into())
        .build_stateless()
        .actions()
        .len(),
      2
    );
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
    let stateless = LexerBuilder::<MyKind>::default()
      .append_default_with(|a| a.from(word("A")).into())
      .build_stateless();
    assert_eq!(stateless.actions().len(), 1);
    assert_eq!(stateless.actions()[0].possible_kinds().len(), 1);
    assert!(stateless.actions()[0]
      .possible_kinds()
      .contains(&Anonymous.id()),);

    // many
    let stateless = LexerBuilder::<MyKind>::default()
      .append_default_with(|a| [a.from(word("A")), a.from(word("B"))].into())
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
  }
}
