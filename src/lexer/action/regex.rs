use super::{output::ActionOutput, Action};
use regex::Regex;
use std::collections::HashSet;

// TODO: only in feature `regex`
impl<ActionState, ErrorType> Action<(), ActionState, ErrorType> {
  pub fn regex(re: &str) -> Result<Self, regex::Error> {
    let regex = Regex::new(re)?;
    Ok(Action {
      possible_kinds: HashSet::new(),
      maybe_muted: false,
      exec: Box::new(move |input| match regex.find(input.rest()) {
        Some(m) => Some(ActionOutput {
          kind: (),
          digested: m.end(),
          muted: false,
          error: None,
        }),
        None => None,
      }),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::input::ActionInput;

  #[test]
  fn regex_start() {
    let mut state = ();
    let action: Action<(), (), _> = Action::regex(r"^\d+").unwrap();
    let mut input = ActionInput::new("123", 0, &mut state, false);
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
    let mut input = ActionInput::new("abc123", 3, &mut state, false);
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
