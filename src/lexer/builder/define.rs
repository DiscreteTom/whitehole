use super::{ActionList, LexerBuilder};
use crate::lexer::{
  action::Action,
  token::{MockTokenKind, SubTokenKind, TokenKindIdBinding},
};

impl<Kind, ActionState, ErrorType> LexerBuilder<TokenKindIdBinding<Kind>, ActionState, ErrorType> {
  /// Define actions and bind them to the provided kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<_>::default();
  /// // append a single action
  /// builder.define(A, word("A"));
  /// # let mut builder = LexerBuilder::<_>::default();
  /// // append multiple actions
  /// builder.define(A, [word("A"), word("AA")]);
  /// ```
  pub fn define<ViaKind>(
    self,
    kind: ViaKind,
    actions: impl Into<ActionList<Action<MockTokenKind<()>, ActionState, ErrorType>>>,
  ) -> Self
  where
    ViaKind:
      SubTokenKind<TokenKindIdBinding<Kind>> + Into<TokenKindIdBinding<Kind>> + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
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
  /// # let mut builder = LexerBuilder::<_, (), i32>::default();
  /// builder.define(A, exact("A").error(123));
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
  /// builder.define_with(A, word("A"), |a| a.error(123));
  /// # let mut builder = LexerBuilder::<_, (), i32>::default();
  /// // append multiple actions
  /// builder.define_with(A, [word("A"), word("B")], |a| a.error(123));
  /// ```
  pub fn define_with<ViaKind>(
    self,
    kind: ViaKind,
    actions: impl Into<ActionList<Action<MockTokenKind<()>, ActionState, ErrorType>>>,
    decorator: impl Fn(
      Action<MockTokenKind<()>, ActionState, ErrorType>,
    ) -> Action<MockTokenKind<()>, ActionState, ErrorType>,
  ) -> Self
  where
    ViaKind:
      SubTokenKind<TokenKindIdBinding<Kind>> + Into<TokenKindIdBinding<Kind>> + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    self.define(kind, Self::map_actions(actions, decorator))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::word;
  use whitehole_macros::_token_kind;

  #[_token_kind]
  #[derive(Clone, Debug)]
  enum MyKind {
    A,
    B,
  }

  #[test]
  fn lexer_builder_define() {
    // single
    let builder = LexerBuilder::<_>::default().define(A, word("A"));
    assert_eq!(builder.actions.len(), 1);
    assert_eq!(builder.actions[0].kind_id(), A::kind_id());

    // multiple
    let builder = LexerBuilder::<_>::default().define(A, [word("A"), word("AA")]);
    assert_eq!(builder.actions.len(), 2);
    assert_eq!(builder.actions[0].kind_id(), A::kind_id());
    assert_eq!(builder.actions[1].kind_id(), A::kind_id());
  }

  #[test]
  fn lexer_builder_define_with() {
    // single
    let builder = LexerBuilder::<_, (), i32>::default().define_with(A, word("A"), |a| a.error(123));
    assert_eq!(builder.actions.len(), 1);
    assert_eq!(builder.actions[0].kind_id(), A::kind_id());
    let stateless = builder.build_stateless();
    assert_eq!(stateless.lex("A").0.token.unwrap().error.unwrap(), 123);

    // multiple
    let builder =
      LexerBuilder::<_, (), i32>::default()
        .define_with(A, [word("A"), word("B")], |a| a.error(123));
    assert_eq!(builder.actions.len(), 2);
    assert_eq!(builder.actions[0].kind_id(), A::kind_id());
    assert_eq!(builder.actions[1].kind_id(), A::kind_id());
    let stateless = builder.build_stateless();
    assert_eq!(stateless.lex("A").0.token.unwrap().error.unwrap(), 123);
    assert_eq!(stateless.lex("B").0.token.unwrap().error.unwrap(), 123);
  }
}
