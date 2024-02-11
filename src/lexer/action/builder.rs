use super::{input::ActionInput, output::ActionOutputWithoutKind, Action};
use std::marker::PhantomData;

/// A helper class to keep track of the generic parameters of [`lexer::Builder`](crate::lexer::Builder).
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
  /// Equals to [`Action::new`](crate::lexer::action::Action::new).
  pub fn new<F>(self, exec: F) -> Action<(), ActionState, ErrorType>
  where
    F: Fn(&mut ActionInput<ActionState>) -> Option<ActionOutputWithoutKind<ErrorType>> + 'static,
  {
    Action::new(exec)
  }

  /// Equals to [`Action::simple`](crate::lexer::action::Action::simple).
  pub fn simple<F>(self, f: F) -> Action<(), ActionState, ErrorType>
  where
    F: Fn(&mut ActionInput<ActionState>) -> usize + 'static,
  {
    Action::simple(f)
  }

  /// Equals to [`Action::regex`](crate::lexer::action::Action::regex).
  pub fn regex(self, re: &str) -> Result<Action<(), ActionState, ErrorType>, regex::Error> {
    Action::regex(re)
  }
}
