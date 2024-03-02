use super::{input::ActionInput, output::ActionOutputWithoutKind, Action};
use std::marker::PhantomData;

/// A helper class to keep track of the generic parameters of [`lexer::Builder`](crate::lexer::builder::LexerBuilder).
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

  /// Return the action as is.
  /// This is useful if you want to re-use existing action (e.g. action utils)
  /// and need to modify it with action decorators.
  /// # Examples
  /// The following code won't pass the compile check
  /// because the compiler can't infer the generic parameter type of `Action`.
  /// ```compile_fail
  /// # use whitehole::lexer::{Action, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// LexerBuilder::<MyKind, i32, i32>::default()
  ///   .define(MyKind::A, Action::exact("A").error(123));
  /// ```
  /// The following code will pass the compile
  /// ```
  /// # use whitehole::lexer::{Action, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// LexerBuilder::<MyKind, i32, i32>::default()
  ///   .define_with(MyKind::A, |a| a.from(Action::exact("A")).error(123));
  /// ```
  pub fn from<Kind>(
    self,
    action: Action<Kind, ActionState, ErrorType>,
  ) -> Action<Kind, ActionState, ErrorType> {
    action
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::output::ActionOutput;

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
  fn action_builder_simple() {
    assert_reject(default_().simple(|_| 0));
    assert_accept_all(default_().simple(|input| input.rest().len()));
  }

  #[test]
  fn action_builder_regex() {
    assert_reject(default_().regex(r"aaa").unwrap());
    assert_accept_all(default_().regex(r"123").unwrap());
  }

  #[test]
  fn action_builder_from() {
    assert_reject(default_().from(Action::regex(r"aaa").unwrap()));
    assert_accept_all(default_().from(Action::regex(r"123").unwrap()));
  }
}
