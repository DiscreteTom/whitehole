use super::{input::ActionInput, output::ActionOutput, Action};

impl<ActionState, ErrorType> Action<(), ActionState, ErrorType> {
  pub fn simple<F>(f: F) -> Self
  where
    F: Fn(&mut ActionInput<ActionState>) -> usize + 'static,
  {
    Action::new(move |input| match f(input) {
      digested if digested > 0 => Some(ActionOutput {
        kind: (),
        digested,
        muted: false,
        error: None,
      }),
      _ => None,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn accept_all() {
    let mut state = ();
    let action = Action::simple(|input| input.buffer().len());
    let mut input = ActionInput::new("123", 0, &mut state, false);
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
    let action = &Action::simple(|input| input.rest().len());
    let mut input = ActionInput::new("123", 1, &mut state, false);
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
    let action = &Action::simple(|_| 0);
    let mut input = ActionInput::new("123", 0, &mut state, false);
    let output: Option<ActionOutput<(), ()>> = action.exec(&mut input);
    assert!(matches!(output, None));
  }
}
