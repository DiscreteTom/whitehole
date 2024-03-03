mod action_list;
mod append;
mod define;
mod ignore;

pub use action_list::*;

use super::{action::Action, stateless::StatelessLexer, token::TokenKind, Lexer};
use std::rc::Rc;

pub struct LexerBuilder<Kind, ActionState = (), ErrorType = ()> {
  actions: Vec<Action<Kind, ActionState, ErrorType>>,
}

impl<Kind, ActionState, ErrorType> Default for LexerBuilder<Kind, ActionState, ErrorType> {
  fn default() -> Self {
    Self {
      actions: Vec::new(),
    }
  }
}
impl<Kind, ActionState, ErrorType> From<Vec<Action<Kind, ActionState, ErrorType>>>
  for LexerBuilder<Kind, ActionState, ErrorType>
{
  fn from(actions: Vec<Action<Kind, ActionState, ErrorType>>) -> Self {
    Self { actions }
  }
}
impl<Kind, ActionState, ErrorType, const N: usize> From<[Action<Kind, ActionState, ErrorType>; N]>
  for LexerBuilder<Kind, ActionState, ErrorType>
{
  fn from(actions: [Action<Kind, ActionState, ErrorType>; N]) -> Self {
    Self {
      actions: actions.into(),
    }
  }
}
impl<
    Kind: TokenKind<Kind> + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
    const N: usize,
  > From<[(Kind, Vec<Action<(), ActionState, ErrorType>>); N]>
  for LexerBuilder<Kind, ActionState, ErrorType>
{
  fn from(actions: [(Kind, Vec<Action<(), ActionState, ErrorType>>); N]) -> Self {
    Self::default().define_from(actions)
  }
}

impl<Kind, ActionState, ErrorType> LexerBuilder<Kind, ActionState, ErrorType> {
  pub fn new() -> Self {
    Self::default()
  }

  fn map_actions<OldKind: 'static, NewKind, F>(
    actions: impl Into<ActionList<Action<OldKind, ActionState, ErrorType>>>,
    f: F,
  ) -> Vec<Action<NewKind, ActionState, ErrorType>>
  where
    F: Fn(Action<OldKind, ActionState, ErrorType>) -> Action<NewKind, ActionState, ErrorType>,
  {
    actions.into().0.into_iter().map(f).collect::<Vec<_>>()
  }

  pub fn build_stateless(self) -> StatelessLexer<Kind, ActionState, ErrorType> {
    StatelessLexer::new(self.actions)
  }

  pub fn build<'text>(self, text: &'text str) -> Lexer<'text, Kind, ActionState, ErrorType>
  where
    ActionState: Default,
  {
    Lexer::with_default_action_state(Rc::new(self.build_stateless()), text)
  }
  // TODO: add build_with
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static>
  Into<StatelessLexer<Kind, ActionState, ErrorType>> for LexerBuilder<Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  fn into(self) -> StatelessLexer<Kind, ActionState, ErrorType> {
    self.build_stateless()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::regex;
  use whitehole_macros::_TokenKind;

  #[derive(_TokenKind, Clone)]
  enum MyKind {
    UnitField,
    // UnnamedField(i32),
    // NamedField { _a: i32 },
  }

  #[test]
  fn append() {
    let mut lexer: Lexer<MyKind, (), ()> = LexerBuilder::default()
      .append_with(|a| a.regex("a+").unwrap().bind(MyKind::UnitField))
      .build("aaa");

    let res = lexer.lex();
    assert_eq!(res.digested, 3);
    assert_eq!(res.errors.len(), 0);
    assert!(res.token.is_some());
    let token = res.token.unwrap();
    assert!(matches!(token.kind, MyKind::UnitField));
    assert_eq!(token.range.start, 0);
    assert_eq!(token.range.end, 3);
    assert_eq!(token.content, "aaa");
    assert!(matches!(token.error, None));
  }

  #[test]
  fn ignore() {
    let mut lexer: Lexer<MyKind, (), ()> = LexerBuilder::default()
      .ignore(regex("a+").unwrap().bind(MyKind::UnitField))
      .build("aaa");

    let res = lexer.lex();
    assert_eq!(res.digested, 3);
    assert_eq!(res.errors.len(), 0);
    assert!(res.token.is_none());

    LexerBuilder::<MyKind, MyState>::default().define_with(MyKind::UnitField, |a| {
      a.regex(r"^\s+")
        .unwrap()
        .prevent(|input| input.state.reject)
    });
  }

  #[derive(Clone, Default)]
  struct MyState {
    pub reject: bool,
  }
}
