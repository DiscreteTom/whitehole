mod common;
pub mod lex;

use super::action::Action;
use std::rc::Rc;

#[derive(Clone)]
pub struct LexerCore<Kind: 'static, ActionState: 'static, ErrorType: 'static> {
  actions: Rc<Vec<Action<Kind, ActionState, ErrorType>>>, // use Rc to make this clone-able
  initial_state: ActionState,
  state: ActionState,
}

impl<Kind, ActionState, ErrorType> LexerCore<Kind, ActionState, ErrorType>
where
  ActionState: Clone,
{
  pub fn new(
    actions: Vec<Action<Kind, ActionState, ErrorType>>,
    initial_state: ActionState,
  ) -> Self {
    LexerCore {
      actions: Rc::new(actions),
      state: initial_state.clone(),
      initial_state,
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
    self.state = self.initial_state.clone();
    self
  }

  pub fn dry_clone(&self) -> Self {
    LexerCore {
      actions: self.actions.clone(),
      initial_state: self.initial_state.clone(),
      state: self.initial_state.clone(), // use the initial state
    }
  }

  pub fn set(&mut self, state: ActionState) -> &mut Self {
    self.state = state;
    self
  }
}
