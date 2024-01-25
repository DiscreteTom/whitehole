mod common;
pub mod lex;
pub mod trim;

use super::{action::Action, token::TokenKind};
use std::rc::Rc;

pub struct LexerCore<Kind: 'static, ActionState: 'static, ErrorType: 'static>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  actions: Rc<Vec<Action<Kind, ActionState, ErrorType>>>, // use Rc to make this clone-able
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static> Clone
  for LexerCore<Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  fn clone(&self) -> Self {
    LexerCore {
      actions: self.actions.clone(),
    }
  }
}

impl<Kind, ActionState, ErrorType> LexerCore<Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  pub fn new(actions: Vec<Action<Kind, ActionState, ErrorType>>) -> Self {
    LexerCore {
      actions: Rc::new(actions),
    }
  }

  // export fields
  pub fn actions(&self) -> &Vec<Action<Kind, ActionState, ErrorType>> {
    &self.actions
  }
}
