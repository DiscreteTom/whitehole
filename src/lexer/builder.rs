use super::{
  action::{builder::ActionBuilder, Action},
  stateless::StatelessLexer,
  token::TokenKind,
  Lexer,
};
use std::rc::Rc;

pub struct Builder<Kind: 'static, ActionState: 'static, ErrorType: 'static>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  actions: Vec<Action<Kind, ActionState, ErrorType>>,
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static> Builder<Kind, ActionState, ErrorType>
where
  Kind: TokenKind + Default + Clone,
  ActionState: Clone + Default,
{
  pub fn ignore_default(self, action: Action<(), ActionState, ErrorType>) -> Self {
    self.ignore(action.bind(Kind::default()))
  }

  pub fn ignore_default_from<F>(self, factory: F) -> Self
  where
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<(), ActionState, ErrorType>,
  {
    self.ignore_default(factory(ActionBuilder::default()))
  }
}

impl<Kind, ActionState, ErrorType> Default for Builder<Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  fn default() -> Self {
    Builder {
      actions: Vec::new(),
    }
  }
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static> Builder<Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  pub fn define<AnyKind>(
    mut self,
    kind: impl Into<Kind>,
    action: Action<AnyKind, ActionState, ErrorType>,
  ) -> Self
  where
    Kind: Clone,
  {
    self.actions.push(action.bind(kind));
    self
  }

  pub fn define_from<AnyKind: 'static, F>(self, kind: impl Into<Kind>, factory: F) -> Self
  where
    Kind: Clone,
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<AnyKind, ActionState, ErrorType>,
  {
    self.define(kind, factory(ActionBuilder::default()))
  }

  pub fn append(mut self, action: Action<Kind, ActionState, ErrorType>) -> Self {
    self.actions.push(action);
    self
  }

  pub fn append_from<F>(self, factory: F) -> Self
  where
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<Kind, ActionState, ErrorType>,
  {
    self.append(factory(ActionBuilder::default()))
  }

  /// Define muted action.
  pub fn ignore(mut self, action: Action<Kind, ActionState, ErrorType>) -> Self {
    self.actions.push(action.mute(true));
    self
  }

  /// Define muted action.
  pub fn ignore_from<F>(self, factory: F) -> Self
  where
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<Kind, ActionState, ErrorType>,
  {
    self.ignore(factory(ActionBuilder::default()))
  }

  pub fn build<'buffer>(
    self,
    buffer: &'buffer str,
  ) -> Lexer<'buffer, Kind, ActionState, ErrorType> {
    Lexer::new(Rc::new(self.build_stateless()), buffer)
  }

  pub fn build_stateless(self) -> StatelessLexer<Kind, ActionState, ErrorType> {
    StatelessLexer::new(self.actions)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use whitehole_macros::TokenKind;

  #[derive(TokenKind, Clone)]
  enum MyKind {
    UnitField,
    // UnnamedField(i32),
    // NamedField { _a: i32 },
  }

  #[test]
  fn append() {
    let mut lexer: Lexer<MyKind, (), ()> = Builder::default()
      .append_from(|a| a.regex("a+").unwrap().bind(MyKind::UnitField))
      .build("aaa");

    let res = lexer.lex();
    assert_eq!(res.digested, 3);
    assert_eq!(res.errors.len(), 0);
    assert!(res.token.is_some());
    let token = res.token.unwrap();
    assert!(matches!(token.kind, MyKind::UnitField));
    assert_eq!(token.range.start, 0);
    assert_eq!(token.range.end, 3);
    assert_eq!(token.content(), "aaa");
    assert!(matches!(token.error, None));
  }

  #[test]
  fn ignore() {
    let mut lexer: Lexer<MyKind, (), ()> = Builder::default()
      .ignore(Action::regex("a+").unwrap().bind(MyKind::UnitField))
      .build("aaa");

    let res = lexer.lex();
    assert_eq!(res.digested, 3);
    assert_eq!(res.errors.len(), 0);
    assert!(res.token.is_none());
  }
}
