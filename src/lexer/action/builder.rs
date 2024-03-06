use super::{input::ActionInput, output::ActionOutputWithoutKind, Action};
use std::marker::PhantomData;

/// A helper class to keep track of the generic parameters of [`lexer::Builder`](crate::lexer::builder::LexerBuilder).
/// # Examples
/// The following code won't pass the compile check
/// because the compiler can't infer the generic parameter type of [`Action`]
/// created by the [`action::exact`](crate::lexer::action::exact) function
/// with the decorator [`error`](Action::error).
/// ```compile_fail
/// # use whitehole::lexer::{Action, LexerBuilder, action::exact};
/// # use whitehole_macros::TokenKind;
/// # #[derive(TokenKind, Clone)]
/// # enum MyKind { A }
/// # let mut builder = LexerBuilder::<MyKind, i32, i32>::default();
/// builder.define(MyKind::A, exact("A").error(123));
/// ```
/// The following code will pass the compile
/// because the variable `a` is an [`ActionBuilder`]
/// and the generic params are inferred from the [`LexerBuilder`](crate::lexer::LexerBuilder).
/// ```
/// # use whitehole::lexer::{Action, LexerBuilder, action::exact};
/// # use whitehole_macros::TokenKind;
/// # #[derive(TokenKind, Clone)]
/// # enum MyKind { A }
/// # let mut builder = LexerBuilder::<MyKind, i32, i32>::default();
/// builder.define_with(MyKind::A, |a| a.from(exact("A")).error(123).into());
/// ```
pub struct ActionBuilder<ActionState, ErrorType> {
  _action_state: PhantomData<ActionState>,
  _error_type: PhantomData<ErrorType>,
}

impl<ActionState, ErrorType> Default for ActionBuilder<ActionState, ErrorType> {
  fn default() -> Self {
    ActionBuilder {
      _action_state: PhantomData,
      _error_type: PhantomData,
    }
  }
}

impl<ActionState, ErrorType> ActionBuilder<ActionState, ErrorType> {
  /// Equals to [`Action::new`](crate::lexer::action::Action::new).
  pub fn new<F>(&self, exec: F) -> Action<(), ActionState, ErrorType>
  where
    F: Fn(&mut ActionInput<ActionState>) -> Option<ActionOutputWithoutKind<Option<ErrorType>>>
      + 'static,
  {
    Action::new(exec)
  }

  /// Return the action as is.
  /// This is useful if you want to re-use existing action (e.g. action utils)
  /// and need to modify it with action decorators.
  pub fn from<Kind>(
    &self,
    action: Action<Kind, ActionState, ErrorType>,
  ) -> Action<Kind, ActionState, ErrorType> {
    action
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{output::ActionOutput, regex::regex};

  fn assert_reject(action: Action<(), (), ()>) {
    let mut action_state = ();
    let mut input = ActionInput::new("123", 0, &mut action_state);
    let output = action.exec(&mut input);
    assert!(matches!(output, None));
  }
  fn assert_accept_all(action: Action<(), (), ()>) {
    let mut action_state = ();
    let mut input = ActionInput::new("123", 0, &mut action_state);
    let output = action.exec(&mut input);
    assert!(matches!(
      output,
      Some(ActionOutput {
        kind: (),
        digested: 3,
        muted: false,
        error: None
      })
    ));
  }
  fn default_() -> ActionBuilder<(), ()> {
    ActionBuilder::default()
  }

  #[test]
  fn action_builder_new() {
    assert_reject(default_().new(|_| None));
    assert_accept_all(default_().new(|input| {
      Some(ActionOutputWithoutKind {
        digested: input.rest().len(),
        muted: false,
        error: None,
      })
    }));
  }

  #[test]
  fn action_builder_from() {
    assert_reject(default_().from(regex(r"aaa").unwrap()));
    assert_accept_all(default_().from(regex(r"123").unwrap()));
  }
}
