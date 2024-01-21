use super::{output::ActionOutput, Action};
use regex::Regex;

// TODO: only in feature `regex`
impl<ActionState, ErrorType> Action<(), ActionState, ErrorType> {
  pub fn regex(re: &str) -> Result<Self, regex::Error> {
    let re = Regex::new(re)?;
    Ok(Action::new(move |input| {
      re.find(input.rest()).map(|m| ActionOutput {
        kind: (),
        digested: m.end(),
        muted: false,
        error: None,
      })
    }))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::{action::input::ActionInput, token::buffer::CowString};

  #[test]
  fn regex_start() {
    let mut state = ();
    let action: Action<(), (), _> = Action::regex(r"^\d+").unwrap();
    let buffer = CowString::new("123");
    let mut input = ActionInput::new(&buffer, 0, &mut state, false);
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
    let buffer = CowString::new("abc123");
    let mut input = ActionInput::new(&buffer, 3, &mut state, false);
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
