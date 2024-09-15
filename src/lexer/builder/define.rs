use super::LexerBuilder;
use crate::{
  lexer::{
    action::Action,
    token::{MockTokenKind, SubTokenKind, TokenKindIdBinding},
  },
  utils::OneOrMore,
};

impl<Kind, State: 'static, Heap: 'static> LexerBuilder<Kind, State, Heap> {
  /// Define actions and bind them to the provided kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, builder::LexerBuilder, token::token_kind};
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
  #[inline]
  pub fn define<ViaKind>(
    self,
    kind: ViaKind,
    actions: impl Into<OneOrMore<Action<MockTokenKind<()>, State, Heap>>>,
  ) -> Self
  where
    ViaKind: SubTokenKind<TokenKind = Kind> + Into<TokenKindIdBinding<Kind>> + Clone + 'static,
  {
    self.append(Self::map_actions(actions, |a| a.bind(kind.clone())))
  }

  /// Define actions with a decorator and bind them to the provided kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, builder::LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A, B }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// // append a single action
  /// builder.define_with(A, word("A"), |a| a.reject());
  /// # let mut builder = LexerBuilder::new();
  /// // append multiple actions
  /// builder.define_with(A, [word("A"), word("B")], |a| a.reject());
  /// # }
  /// ```
  #[inline]
  pub fn define_with<ViaKind>(
    self,
    kind: ViaKind,
    actions: impl Into<OneOrMore<Action<MockTokenKind<()>, State, Heap>>>,
    decorator: impl Fn(Action<MockTokenKind<()>, State, Heap>) -> Action<MockTokenKind<()>, State, Heap>,
  ) -> Self
  where
    ViaKind: SubTokenKind<TokenKind = Kind> + Into<TokenKindIdBinding<Kind>> + Clone + 'static,
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
