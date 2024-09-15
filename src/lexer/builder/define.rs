use super::LexerBuilder;
use crate::{
  kind::{KindIdBinding, MockKind, SubKind},
  lexer::action::Action,
  utils::OneOrMore,
};

impl<'a, Kind, State: 'a, Heap: 'a> LexerBuilder<'a, Kind, State, Heap> {
  /// Define actions and bind them to the provided kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, builder::LexerBuilder, token::kind};
  /// # #[whitehole_kind]
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
    actions: impl Into<OneOrMore<Action<'a, MockKind<()>, State, Heap>>>,
  ) -> Self
  where
    ViaKind: SubKind<Kind = Kind> + Into<KindIdBinding<Kind>> + Clone + 'a,
  {
    self.append(Self::map_actions(actions, move |a| a.bind(kind.clone())))
  }

  /// Define actions with a decorator and bind them to the provided kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, builder::LexerBuilder, token::kind};
  /// # #[whitehole_kind]
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
    actions: impl Into<OneOrMore<Action<'a, MockKind<()>, State, Heap>>>,
    decorator: impl Fn(Action<MockKind<()>, State, Heap>) -> Action<MockKind<()>, State, Heap> + 'a,
  ) -> Self
  where
    ViaKind: SubKind<Kind = Kind> + Into<KindIdBinding<Kind>> + Clone + 'a,
  {
    self.define(kind, Self::map_actions(actions, decorator))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{exact, word};
  use whitehole_macros::_whitehole_kind;

  #[_whitehole_kind]
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
