use super::{input::ActionInput, output::ActionOutputWithoutKind, Action};
use std::marker::PhantomData;

pub struct ActionBuilder<ActionState: 'static, ErrorType: 'static> {
  _action_state: PhantomData<ActionState>,
  _error_type: PhantomData<ErrorType>,
}

impl<ActionState: 'static, ErrorType: 'static> Default for ActionBuilder<ActionState, ErrorType> {
  fn default() -> Self {
    ActionBuilder {
      _action_state: PhantomData,
      _error_type: PhantomData,
    }
  }
}

impl<ActionState, ErrorType> ActionBuilder<ActionState, ErrorType> {
  pub fn new<F>(self, exec: F) -> Action<(), ActionState, ErrorType>
  where
    F: Fn(&mut ActionInput<ActionState>) -> Option<ActionOutputWithoutKind<ErrorType>> + 'static,
  {
    Action::new(exec)
  }

  pub fn simple<F>(self, f: F) -> Action<(), ActionState, ErrorType>
  where
    F: Fn(&mut ActionInput<ActionState>) -> usize + 'static,
  {
    Action::simple(f)
  }

  pub fn regex(self, re: &str) -> Result<Action<(), ActionState, ErrorType>, regex::Error> {
    Action::regex(re)
  }
}
