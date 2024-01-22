mod common;
pub mod lex;

use super::{action::Action, token::TokenKind};
use std::rc::Rc;

#[derive(Clone)]
pub struct LexerCore<Kind: 'static, ActionState: 'static, ErrorType: 'static>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  actions: Rc<Vec<Action<Kind, ActionState, ErrorType>>>, // use Rc to make this clone-able
  state: ActionState,
}

impl<Kind, ActionState, ErrorType> LexerCore<Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  pub fn new(actions: Vec<Action<Kind, ActionState, ErrorType>>) -> Self {
    LexerCore {
      actions: Rc::new(actions),
      state: ActionState::default(),
    }
  }

  // export fields
  pub fn actions(&self) -> &Vec<Action<Kind, ActionState, ErrorType>> {
    &self.actions
  }
  pub fn state(&self) -> &ActionState {
    &self.state
  }
  pub fn state_mut(&mut self) -> &mut ActionState {
    &mut self.state
  }

  pub fn reset(&mut self) -> &mut Self {
    self.state = ActionState::default();
    self
  }

  pub fn dry_clone(&self) -> Self {
    LexerCore {
      actions: self.actions.clone(),
      state: ActionState::default(),
    }
  }

  pub fn set(&mut self, state: ActionState) -> &mut Self {
    self.state = state;
    self
  }
}
