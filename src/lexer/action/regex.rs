// TODO: only available in feature `regex`
use super::{simple::simple, Action};
use crate::lexer::token::MockTokenKind;
use regex::{Error, Regex};

/// Create a new action that uses a regex to match the rest of input.
///
/// It's recommended to set [`Action::head_matcher`] to optimize the lex performance.
/// # Examples
/// Usually the regex should start with `^` to match from the start of the rest of the input.
/// ```
/// # use whitehole::lexer::action::{Action, regex};
/// # let action: Action<_, (), ()> =
/// regex(r"^\d+").unwrap();
/// ```
/// Set [`Action::head_matcher`]:
/// ```
/// # use whitehole::lexer::action::Action;
/// # use whitehole::lexer::action::regex;
/// # use whitehole::lexer::action::HeadMatcher;
/// # use std::collections::HashSet;
/// # let action: Action<_> =
/// regex(r"^abc").unwrap().unchecked_head_in(['a']);
/// # let action: Action<_> =
/// regex(r"^\d+").unwrap().unchecked_head_in_range('0'..='9');
/// ```
pub fn regex<ActionState, ErrorType>(
  re: &str,
) -> Result<Action<MockTokenKind<()>, ActionState, ErrorType>, Error> {
  // TODO: unwrap the result, add regex_try
  Regex::new(re).map(|re| simple(move |input| re.find(input.rest()).map(|m| m.len()).unwrap_or(0)))
}

// TODO: add this function when there is a need / good use case
/// Create a new action that uses a regex to match the rest of input and transform the captures.
// pub fn regex_captures<ActionState, ErrorType, T, F>(
//   re: &str,
//   capture_transformer: F,
// ) -> Result<Action<MockTokenKind<T>, ActionState, ErrorType>, Error>
// where
//   F: Fn(Captures) -> T + 'static,
// {
//   Regex::new(re).map(|re| {
//     simple_option_with_data(move |input| {
//       re.captures(input.rest())
//         .map(|captures| (captures[0].len(), capture_transformer(captures)))
//     })
//   })
// }

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::{
    action::{input::ActionInput, output::ActionOutput},
    token::{TokenKindId, TokenKindIdProvider},
  };

  #[test]
  fn match_at_start() {
    let action: Action<MockTokenKind<()>> = regex(r"^\d+").unwrap();
    assert!(matches!(
      action.exec(&mut ActionInput::new("123", 0, &mut ())),
      Some(ActionOutput {
        kind: mock,
        digested: 3,
        muted: false,
        error: None,
      }) if matches!(mock.data, ()) && mock.id() == &TokenKindId::new(0)
    ));
  }

  #[test]
  fn match_at_middle() {
    let action: Action<MockTokenKind<()>> = regex(r"^\d+").unwrap();
    assert!(matches!(
      action.exec(&mut ActionInput::new("abc123", 3, &mut ())),
      Some(ActionOutput {
        kind: mock,
        digested: 3,
        muted: false,
        error: None,
      }) if matches!(mock.data, ()) && mock.id() == &TokenKindId::new(0)
    ));
  }
}
