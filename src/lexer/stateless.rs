mod common;
pub mod lex;
pub mod trim;

use super::{action::Action, token::TokenKind};
use std::rc::Rc;

/// Stateless, immutable lexer.
pub struct StatelessLexer<Kind: 'static, ActionState: 'static, ErrorType: 'static>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  // use Rc to make StatelessLexer clone-able
  // so that user can use `lexer.stateless.clone()` to create a new stateless lexer with little cost
  actions: Rc<Vec<Action<Kind, ActionState, ErrorType>>>,
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static> Clone
  for StatelessLexer<Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  fn clone(&self) -> Self {
    StatelessLexer {
      actions: self.actions.clone(),
    }
  }
}

impl<Kind, ActionState, ErrorType> StatelessLexer<Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  pub fn new(actions: Vec<Action<Kind, ActionState, ErrorType>>) -> Self {
    StatelessLexer {
      actions: Rc::new(actions),
    }
  }

  // export fields
  pub fn actions(&self) -> &Vec<Action<Kind, ActionState, ErrorType>> {
    &self.actions
  }
}
