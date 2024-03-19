mod action_list;
mod append;
mod define;
mod ignore;

pub use action_list::*;

use super::{
  action::{Action, ActionInputRestHeadMatcher},
  stateless::{ActionHeadMap, StatelessLexer},
  token::TokenKind,
  Lexer,
};
use std::{collections::HashMap, rc::Rc};

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
  > From<[(Kind, ActionList<Action<(), ActionState, ErrorType>>); N]>
  for LexerBuilder<Kind, ActionState, ErrorType>
{
  fn from(actions: [(Kind, ActionList<Action<(), ActionState, ErrorType>>); N]) -> Self {
    Self::default().define_from(actions)
  }
}

impl<Kind, ActionState, ErrorType> LexerBuilder<Kind, ActionState, ErrorType> {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn build_stateless_from(
    actions: Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
  ) -> StatelessLexer<Kind, ActionState, ErrorType> {
  }

  // TODO: move into `generate`?
  pub fn build_stateless(self) -> StatelessLexer<Kind, ActionState, ErrorType> {
    // wrap actions with Rc, make them immutable and clone-able
    Self::build_stateless_from(self.actions.into_iter().map(Rc::new).collect::<Vec<_>>())
  }

  pub fn build_with<'text>(
    self,
    action_state: ActionState,
    text: &'text str,
  ) -> Lexer<'text, Kind, ActionState, ErrorType> {
    Lexer::new(Rc::new(self.build_stateless()), action_state, text)
  }

  pub fn build<'text>(self, text: &'text str) -> Lexer<'text, Kind, ActionState, ErrorType>
  where
    ActionState: Default,
  {
    self.build_with(ActionState::default(), text)
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
}

impl<Kind, ActionState, ErrorType> Into<StatelessLexer<Kind, ActionState, ErrorType>>
  for LexerBuilder<Kind, ActionState, ErrorType>
{
  fn into(self) -> StatelessLexer<Kind, ActionState, ErrorType> {
    self.build_stateless()
  }
}
impl<Kind, ActionState, ErrorType> From<Vec<Action<Kind, ActionState, ErrorType>>>
  for StatelessLexer<Kind, ActionState, ErrorType>
{
  fn from(actions: Vec<Action<Kind, ActionState, ErrorType>>) -> Self {
    LexerBuilder::from(actions).into()
  }
}
impl<Kind, ActionState, ErrorType> From<Vec<Rc<Action<Kind, ActionState, ErrorType>>>>
  for StatelessLexer<Kind, ActionState, ErrorType>
{
  fn from(actions: Vec<Rc<Action<Kind, ActionState, ErrorType>>>) -> Self {
    LexerBuilder::build_stateless_from(actions)
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

  #[derive(Clone, Default)]
  struct MyState {
    pub reject: bool,
  }
}
