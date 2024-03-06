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
  fn accept_all() {
    let mut state = ();
    let action = simple(|input| input.text().len());
    let mut input = ActionInput::new("123", 0, &mut state);
    let output = action.exec(&mut input);

    assert!(matches!(output, Some { .. }));
    if let Some(ActionOutput {
      kind,
      digested,
      muted,
      error,
    }) = output
    {
      assert_eq!(kind, ());
      assert_eq!(digested, 3);
      assert_eq!(muted, false);
      assert_eq!(error, None::<()>);
    }
  }

  #[test]
  fn accept_rest() {
    let mut state = ();
    let action = &simple(|input| input.rest().len());
    let mut input = ActionInput::new("123", 1, &mut state);
    let output = action.exec(&mut input);
    assert!(matches!(output, Some { .. }));
    if let Some(ActionOutput {
      kind,
      digested,
      muted,
      error,
    }) = output
    {
      assert_eq!(kind, ());
      assert_eq!(digested, 2);
      assert_eq!(muted, false);
      assert_eq!(error, None::<()>);
    }
  }

  #[test]
  fn reject() {
    let mut state = ();
    let action = &simple(|_| 0);
    let mut input = ActionInput::new("123", 0, &mut state);
    let output: Option<ActionOutput<(), Option<()>>> = action.exec(&mut input);
    assert!(matches!(output, None));
  }

  #[test]
  fn action_builder_simple() {
    let action: Action<()> = ActionBuilder::default().simple(|input| input.rest().len());
    let mut state = ();
    let mut input = ActionInput::new("123", 1, &mut state);
    let output = action.exec(&mut input);
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
