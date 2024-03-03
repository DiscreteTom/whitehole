use super::{ActionList, LexerBuilder};
use crate::lexer::{action::ActionBuilder, token::TokenKind, Action};

impl<Kind, ActionState, ErrorType> LexerBuilder<Kind, ActionState, ErrorType> {
  /// Define actions and bind them to the provided kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append a single action
  /// builder.define(A, word("A"));
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append multiple actions
  /// builder.define(A, [word("A"), word("AA")]);
  /// ```
  pub fn define(
    self,
    kind: impl Into<Kind>,
    actions: impl Into<ActionList<Action<(), ActionState, ErrorType>>>,
  ) -> Self
  where
    Kind: TokenKind<Kind> + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    let kind = kind.into();
    self.append(Self::map_actions(actions, |a| a.bind(kind.clone())))
  }

  /// Define actions with [`ActionBuilder`] and bind it to the provided kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append a single action
  /// builder.define_with(A, |a| a.from(word("A")).into());
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append multiple actions
  /// builder.define_with(A, |a| [
  ///   a.from(word("A")),
  ///   a.from(word("AA"))
  /// ].into());
  /// ```
  pub fn define_with<F>(self, kind: impl Into<Kind>, factory: F) -> Self
  where
    Kind: TokenKind<Kind> + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: FnOnce(
      ActionBuilder<ActionState, ErrorType>,
    ) -> ActionList<Action<(), ActionState, ErrorType>>,
  {
    self.define(kind, factory(ActionBuilder::default()))
  }

  /// Define actions and bind them to the provided kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define_from([
  ///   (A, word("A").into()), // append a single action
  ///   (B, [word("B"), word("BB")].into()), // append multiple actions
  /// ]);
  /// ```
  pub fn define_from<const N: usize>(
    self,
    defs: [(
      impl Into<Kind>,
      ActionList<Action<(), ActionState, ErrorType>>,
    ); N],
  ) -> Self
  where
    Kind: TokenKind<Kind> + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    defs.into_iter().fold(self, |builder, (kind, actions)| {
      builder.define(kind, actions)
    })
  }
}
