use super::{builder::ActionBuilder, input::ActionInput, output::ActionOutputWithoutKind, Action};

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

impl<ActionState, ErrorType> ActionBuilder<ActionState, ErrorType> {
  /// Equals to [`simple`](crate::lexer::action::simple::simple).
  pub fn simple<F>(self, f: F) -> Action<(), ActionState, ErrorType>
  where
    F: Fn(&mut ActionInput<ActionState>) -> usize + 'static,
  {
    simple(f)
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
    let output: Option<ActionOutput<(), ()>> = action.exec(&mut input);
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
