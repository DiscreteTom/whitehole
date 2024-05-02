// TODO: only available in feature `regex`
use super::{simple::simple, SubAction};
use regex::Regex;

/// Create a new sub action that uses a regex to match the rest of input.
/// Reject if the regex does not match anything.
/// # Panics
/// Panics if the regex is invalid.
/// # Examples
/// Usually the regex should start with `^` to match from the start of the rest of the input.
/// ```
/// use whitehole::lexer::action::{SubAction, regex};
/// let a: SubAction<()> = regex(r"^\d+");
/// ```
// TODO: to distinguish between `regex` crate and this function, rename to `re`.
pub fn regex<ActionState>(re: &str) -> SubAction<ActionState> {
  Regex::new(re)
    .map(|re| simple(move |input| re.find(input.rest()).map(|m| m.len()).unwrap_or(0)))
    .unwrap()
}

// since `regex` is based on `simple`, users could create their own regex action
// like `regex_capture` or `regex_try`. if there are other commonly used regex actions, add them here.

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::SubActionInput;

  #[test]
  fn match_at_start() {
    let action: SubAction<()> = regex(r"^\d+");
    assert!(matches!(
      action.exec(&SubActionInput::new("123", 0, &mut ()).unwrap()),
      Some(3)
    ));
  }

  #[test]
  fn match_at_middle() {
    let action: SubAction<()> = regex(r"^\d+");
    assert!(matches!(
      action.exec(&SubActionInput::new("abc123", 3, &mut ()).unwrap()),
      Some(3)
    ));
  }
}
