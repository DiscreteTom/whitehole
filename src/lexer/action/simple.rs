use super::{input::ActionInput, output::ActionOutput, Action};

impl<ActionState, ErrorType> Action<(), ActionState, ErrorType> {
  pub fn simple<F>(f: F) -> Self
  where
    F: Fn(&mut ActionInput<ActionState>) -> usize + 'static,
  {
    Action {
      exec: Box::new(move |input| match f(input) {
        digested if digested > 0 => Some(ActionOutput {
          kind: (),
          digested,
          muted: false,
          error: None,
        }),
        _ => return None,
      }),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn accept_all() {
    let output = Action::simple(|input| input.buffer().len()).exec(&mut ActionInput::new(
      "123",
      0,
      &mut (),
      false,
    ));
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
    let output = Action::simple(|input| input.rest().len()).exec(&mut ActionInput::new(
      "123",
      1,
      &mut (),
      false,
    ));
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
    let output: Option<ActionOutput<(), ()>> =
      Action::simple(|_| 0).exec(&mut ActionInput::new("123", 0, &mut (), false));
    assert!(matches!(output, None));
  }
}
