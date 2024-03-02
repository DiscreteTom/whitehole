use super::Action;
use regex::Regex;

// TODO: only in feature `regex`
impl<ActionState, ErrorType> Action<(), ActionState, ErrorType> {
  pub fn regex(re: &str) -> Result<Self, regex::Error> {
    Regex::new(re)
      .map(|re| Action::simple(move |input| re.find(input.rest()).map(|m| m.len()).unwrap_or(0)))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{input::ActionInput, output::ActionOutput};

  #[test]
  fn regex_start() {
    let mut state = ();
    let action: Action<(), (), _> = Action::regex(r"^\d+").unwrap();
    let mut input = ActionInput::new("123", 0, &mut state);
    let output: Option<ActionOutput<(), _>> = action.exec(&mut input);
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
    let mut state = ();
    let action = Action::regex(r"^\d+").unwrap();
    let mut input = ActionInput::new("abc123", 3, &mut state);
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
}
