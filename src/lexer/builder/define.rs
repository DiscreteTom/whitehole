use super::{ActionList, LexerBuilder};
use crate::lexer::{token::TokenKind, Action};

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

  /// Define actions with a decorator and bind them to the provided kind.
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
  /// builder.define(A, exact("A").error(123));
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
  /// builder.define_with(A, word("A"), |a| a.error(123));
  /// # let mut builder = LexerBuilder::<MyKind, (), i32>::default();
  /// // append multiple actions
  /// builder.define_with(A, [word("A"), word("B")], |a| a.error(123));
  /// ```
  pub fn define_with<F>(
    self,
    kind: impl Into<Kind>,
    actions: impl Into<ActionList<Action<(), ActionState, ErrorType>>>,
    decorator: F,
  ) -> Self
  where
    Kind: TokenKind<Kind> + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(Action<(), ActionState, ErrorType>) -> Action<(), ActionState, ErrorType>,
  {
    self.define(kind, Self::map_actions(actions, decorator))
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::word;
  use whitehole_macros::_TokenKind;
  use MyKind::*;

  #[derive(_TokenKind, Clone)]
  enum MyKind {
    A,
    B,
  }

  #[test]
  fn lexer_builder_define() {
    // single
    let stateless = LexerBuilder::<MyKind>::default()
      .define(A, word("A"))
      .build_stateless();
    assert_eq!(stateless.actions().len(), 1);
    assert_eq!(stateless.actions()[0].possible_kinds().len(), 1);
    assert!(stateless.actions()[0].possible_kinds().contains(&A.id()));

    // multiple
    let stateless = LexerBuilder::<MyKind>::default()
      .define(A, [word("A"), word("AA")])
      .build_stateless();
    assert_eq!(stateless.actions().len(), 2);
    assert_eq!(stateless.actions()[0].possible_kinds().len(), 1);
    assert!(stateless.actions()[0].possible_kinds().contains(&A.id()));
    assert_eq!(stateless.actions()[1].possible_kinds().len(), 1);
    assert!(stateless.actions()[1].possible_kinds().contains(&A.id()));
  }

  #[test]
  fn lexer_builder_define_with() {
    // single
    let stateless = LexerBuilder::<MyKind, (), i32>::default()
      .define_with(A, word("A"), |a| a.error(123))
      .build_stateless();
    assert_eq!(stateless.actions().len(), 1);
    assert_eq!(stateless.actions()[0].possible_kinds().len(), 1);
    assert!(stateless.actions()[0].possible_kinds().contains(&A.id()));
    assert_eq!(stateless.lex("A").0.token.unwrap().error.unwrap(), 123);

    // multiple
    let stateless = LexerBuilder::<MyKind, (), i32>::default()
      .define_with(A, [word("A"), word("B")], |a| a.error(123))
      .build_stateless();
    assert_eq!(stateless.actions().len(), 2);
    assert_eq!(stateless.actions()[0].possible_kinds().len(), 1);
    assert!(stateless.actions()[0].possible_kinds().contains(&A.id()));
    assert_eq!(stateless.actions()[1].possible_kinds().len(), 1);
    assert!(stateless.actions()[1].possible_kinds().contains(&A.id()));
    assert_eq!(stateless.lex("A").0.token.unwrap().error.unwrap(), 123);
    assert_eq!(stateless.lex("B").0.token.unwrap().error.unwrap(), 123);
  }

  #[test]
  fn lexer_builder_define_from() {
    let stateless = LexerBuilder::<MyKind>::default()
      .define_from([
        (A, word("A").into()),               // single
        (B, [word("B"), word("BB")].into()), // multiple
      ])
      .build_stateless();
    assert_eq!(stateless.actions().len(), 3);
    assert_eq!(stateless.actions()[0].possible_kinds().len(), 1);
    assert!(stateless.actions()[0].possible_kinds().contains(&A.id()));
    assert_eq!(stateless.actions()[1].possible_kinds().len(), 1);
    assert!(stateless.actions()[1].possible_kinds().contains(&B.id()));
    assert_eq!(stateless.actions()[2].possible_kinds().len(), 1);
    assert!(stateless.actions()[2].possible_kinds().contains(&B.id()));
  }
}
