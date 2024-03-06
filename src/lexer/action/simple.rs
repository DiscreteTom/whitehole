use super::{
  builder::ActionBuilder, input::ActionInput, output::ActionOutputWithoutKind, Action, ActionOutput,
};
use crate::lexer::token::MockTokenKind;

/// Provide a function that digests the rest of the input text and returns the number of digested characters.
/// Return `0` if the action is rejected.
pub fn simple<ActionState, ErrorType, F>(f: F) -> Action<(), ActionState, ErrorType>
where
  F: Fn(&mut ActionInput<ActionState>) -> usize + 'static,
{
  Action::new(move |input| match f(input) {
    digested if digested > 0 => Some(ActionOutputWithoutKind {
      digested,
      muted: false,
      error: None,
    }),
    _ => None,
  })
}

/// Provide a function that digests the rest of the input text and returns the number of digested characters.
/// `0` is allowed as an accepted number of digested characters.
/// Return `None` if the action is rejected.
pub fn simple_option<ActionState, ErrorType, F>(f: F) -> Action<(), ActionState, ErrorType>
where
  F: Fn(&mut ActionInput<ActionState>) -> Option<usize> + 'static,
{
  Action::new(move |input| {
    f(input).map(|digested| ActionOutputWithoutKind {
      digested,
      muted: false,
      error: None,
    })
  })
}

/// Provide a function that digests the rest of the input text,
/// returns the number of digested characters and the data.
/// `0` is allowed as an accepted number of digested characters.
/// Return `None` if the action is rejected.
pub fn simple_option_with_data<T, ActionState, ErrorType, F>(
  f: F,
) -> Action<MockTokenKind<T>, ActionState, ErrorType>
where
  F: Fn(&mut ActionInput<ActionState>) -> Option<(usize, T)> + 'static,
{
  Action::with_data(move |input| {
    f(input).map(|(digested, data)| {
      ActionOutput {
        kind: MockTokenKind { data },
        digested,
        muted: false,
        error: None,
      }
      .into()
    })
  })
}

impl<ActionState, ErrorType> ActionBuilder<ActionState, ErrorType> {
  /// Equals to [`action::simple`](crate::lexer::action::simple::simple).
  pub fn simple<F>(&self, f: F) -> Action<(), ActionState, ErrorType>
  where
    F: Fn(&mut ActionInput<ActionState>) -> usize + 'static,
  {
    simple(f)
  }
  /// Equals to [`action::simple_option`](crate::lexer::action::simple::simple_option).
  pub fn simple_option<F>(&self, f: F) -> Action<(), ActionState, ErrorType>
  where
    F: Fn(&mut ActionInput<ActionState>) -> Option<usize> + 'static,
  {
    simple_option(f)
  }
  /// Equals to [`action::simple_option_with_data`](crate::lexer::action::simple::simple_option_with_data).
  pub fn simple_option_with_data<T, F>(
    &self,
    f: F,
  ) -> Action<MockTokenKind<T>, ActionState, ErrorType>
  where
    F: Fn(&mut ActionInput<ActionState>) -> Option<(usize, T)> + 'static,
  {
    simple_option_with_data(f)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::output::ActionOutput;

  #[test]
  fn simple_accept_all() {
    let action: Action<()> = simple(|input| input.text().len());
    let output = action.exec(&mut ActionInput::new("123", 0, &mut ()));

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

  #[test]
  fn simple_accept_rest() {
    let action: Action<()> = simple(|input| input.rest().len());
    let output = action.exec(&mut ActionInput::new("123", 1, &mut ()));
    assert!(matches!(
      output,
      Some(ActionOutput {
        kind: (),
        digested: 2,
        muted: false,
        error: None
      })
    ));
  }

  #[test]
  fn reject() {
    let action: Action<()> = simple(|_| 0);
    let output = action.exec(&mut ActionInput::new("123", 0, &mut ()));
    assert!(matches!(output, None));
  }

  #[test]
  fn action_builder_simple() {
    let action: Action<()> = ActionBuilder::default().simple(|input| input.rest().len());
    let output = action.exec(&mut ActionInput::new("123", 1, &mut ()));
    assert!(matches!(
      output,
      Some(ActionOutput {
        kind: (),
        digested: 2,
        muted: false,
        error: None
      })
    ));
  }
}
