use super::{ActionList, LexerBuilder};
use crate::lexer::{action::ActionBuilder, token::TokenKind, Action};

impl<Kind, ActionState, ErrorType> LexerBuilder<Kind, ActionState, ErrorType> {
  /// Define [`muted`](Action::maybe_muted) actions by calling [`Action::mute`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append a single action
  /// builder.ignore(word("A").bind(A));
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

  /// Define [`muted`](Action::maybe_muted) actions by calling [`Action::mute`] with [`ActionBuilder`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append a single action
  /// builder.ignore_with(|a| a.from(word("A")).bind(A).into);
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append multiple actions
  /// builder.ignore_with(|a| [
  ///   a.from(word("A")).bind(A),
  ///   a.from(word("B")).bind(B)
  /// ].into());
  /// ```
  pub fn ignore_with<F>(self, factory: F) -> Self
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: FnOnce(
      ActionBuilder<ActionState, ErrorType>,
    ) -> ActionList<Action<Kind, ActionState, ErrorType>>,
  {
    self.ignore(factory(ActionBuilder::default()))
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

  /// Define [`muted`](Action::maybe_muted) actions by calling [`Action::mute`] with [`ActionBuilder`] and bind it to the default kind.
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
  /// builder.ignore_default_with(|a| a.from(whitespaces()));
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append multiple actions
  /// builder.ignore_default_with(|a| [
  ///   a.from(whitespaces()),
  ///   a.from(word("_"))
  /// ].into());
  /// ```
  pub fn ignore_default_with<F>(self, factory: F) -> Self
  where
    Kind: TokenKind<Kind> + Default + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: FnOnce(
      ActionBuilder<ActionState, ErrorType>,
    ) -> ActionList<Action<(), ActionState, ErrorType>>,
  {
    self.ignore_default(factory(ActionBuilder::default()))
  }
}
