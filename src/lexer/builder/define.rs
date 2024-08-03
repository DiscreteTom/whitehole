use super::LexerBuilder;
use crate::{
  lexer::{
    action::Action,
    token::{MockTokenKind, SubTokenKind, TokenKindIdBinding},
  },
  utils::OneOrMore,
};

impl<Kind, State, ErrorType> LexerBuilder<Kind, State, ErrorType> {
  /// Define actions and bind them to the provided kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A, B }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// // append a single action
  /// builder.define(A, word("A"));
  /// # let mut builder = LexerBuilder::new();
  /// // append multiple actions
  /// builder.define(A, [word("A"), word("AA")]);
  /// # }
  /// ```
  pub fn define<ViaKind>(
    self,
    kind: ViaKind,
    actions: impl Into<OneOrMore<Action<MockTokenKind<()>, State, ErrorType>>>,
  ) -> Self
  where
    ViaKind: SubTokenKind<TokenKind = Kind> + Into<TokenKindIdBinding<Kind>> + Clone + 'static,
    State: 'static,
    ErrorType: 'static,
  {
    self.append(Self::map_actions(actions, |a| a.bind(kind.clone())))
  }

  /// Define actions with a decorator and bind them to the provided kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A, B }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::with_error();
  /// // append a single action
  /// builder.define_with(A, word("A"), |a| a.error(123));
  /// # let mut builder = LexerBuilder::with_error();
  /// // append multiple actions
  /// builder.define_with(A, [word("A"), word("B")], |a| a.error(123));
  /// # }
  /// ```
  pub fn define_with<ViaKind>(
    self,
    kind: ViaKind,
    actions: impl Into<OneOrMore<Action<MockTokenKind<()>, State, ErrorType>>>,
    decorator: impl Fn(
      Action<MockTokenKind<()>, State, ErrorType>,
    ) -> Action<MockTokenKind<()>, State, ErrorType>,
  ) -> Self
  where
    ViaKind: SubTokenKind<TokenKind = Kind> + Into<TokenKindIdBinding<Kind>> + Clone + 'static,
    State: 'static,
    ErrorType: 'static,
  {
    self.define(kind, Self::map_actions(actions, decorator))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{exact, word};
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
    let builder = LexerBuilder::new().define(A, word("A"));
    assert_eq!(builder.actions.len(), 1);
    assert_eq!(builder.actions[0].kind(), A::kind_id());

    // multiple
    let builder = LexerBuilder::new().define(A, [word("A"), word("AA")]);
    assert_eq!(builder.actions.len(), 2);
    assert_eq!(builder.actions[0].kind(), A::kind_id());
    assert_eq!(builder.actions[1].kind(), A::kind_id());
  }

  #[test]
  fn lexer_builder_define_with() {
    // single
    let builder = LexerBuilder::new().define_with(A, word("A"), |a| a.mute());
    assert_eq!(builder.actions.len(), 1);
    assert_eq!(builder.actions[0].kind(), A::kind_id());
    assert!(builder.actions[0].muted());

    // multiple
    let builder = LexerBuilder::new().define_with(A, [exact("A"), exact("B")], |a| a.mute());
    assert_eq!(builder.actions.len(), 2);
    assert_eq!(builder.actions[0].kind(), A::kind_id());
    assert_eq!(builder.actions[1].kind(), A::kind_id());
    assert!(builder.actions[0].muted());
    assert!(builder.actions[1].muted());
  }
}
