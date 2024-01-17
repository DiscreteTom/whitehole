use super::{output::ActionOutput, Action};
use regex::Regex;

impl<ActionState, ErrorType> Action<(), ActionState, ErrorType> {
  pub fn regex(re: &str) -> Self {
    let regex = Regex::new(re).unwrap();
    Action {
      exec: Box::new(move |input| match regex.find(input.rest()) {
        Some(m) => Some(ActionOutput {
          kind: (),
          digested: m.end(),
          muted: false,
          error: None,
        }),
        None => None,
      }),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::input::ActionInput;

  #[test]
  fn regex_start() {
    let action: Action<(), (), _> = Action::regex(r"^\d+");
    let output: Option<ActionOutput<(), _>> =
      (action.exec)(&mut ActionInput::new("123", 0, &mut (), false));
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
  fn regex_middle() {
    let output =
      (Action::regex(r"^\d+").exec)(&mut (ActionInput::new("abc123", 3, &mut (), false)));
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
}
