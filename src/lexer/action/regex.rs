// TODO: only available in feature `regex`

use super::{simple::simple, Action};
use crate::lexer::token::MockTokenKind;
use regex::Regex;

/// Create a new action that uses a regex to match the rest of input.
/// Reject if the regex does not match anything.
///
/// It's recommended to set [`Action::head_matcher`] to optimize the lex performance.
/// # Panics
/// Panics if the regex is invalid.
/// # Examples
/// Usually the regex should start with `^` to match from the start of the rest of the input.
/// You can use `r"..."` to create a raw string to avoid escaping.
/// ```
/// use whitehole::lexer::action::{Action, regex};
/// let a: Action<_> = regex(r"^\d+");
/// ```
// TODO: to distinguish between `regex` crate and this function, rename to `re`.
pub fn regex<ActionState, ErrorType>(
  re: &str,
) -> Action<MockTokenKind<()>, ActionState, ErrorType> {
  Regex::new(re)
    .map(|re| simple(move |input| re.find(input.rest()).map(|m| m.len()).unwrap_or(0)))
    .unwrap()
}

// since `regex` is based on `simple`, users could create their own regex action
// like `regex_capture` for capture groups or `regex_try` to avoid panic.
// if there are other commonly used regex actions, add them here.

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::ActionInput;

  #[test]
  fn match_at_start() {
    let action: Action<_> = regex(r"^\d+");
    assert!(matches!(
      action
        .exec(&mut ActionInput::new("123", 0, &mut ()).unwrap())
        .unwrap()
        .digested,
      3
    ));
  }

  #[test]
  fn match_at_middle() {
    let action: Action<_> = regex(r"^\d+");
    assert!(matches!(
      action
        .exec(&mut ActionInput::new("abc123", 3, &mut ()).unwrap())
        .unwrap()
        .digested,
      3
    ));
  }
}
