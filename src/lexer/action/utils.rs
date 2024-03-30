mod chars;
mod exact;
mod string_list;
mod word;

pub use chars::*;
pub use exact::*;
pub use string_list::*;
pub use word::*;

use super::{simple::simple, Action, SubAction};
use crate::lexer::token::MockTokenKind;

/// Match from the `open` to the `close`, including the `open` and `close`.
/// If the `close` is not found, accept all the rest.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{SubAction, comment_sub};
/// // single line comment
/// # let action: SubAction<()> =
/// comment_sub("//", "\n");
/// # let action: SubAction<()> =
/// comment_sub("#", "\n");
/// // multi line comment
/// # let action: SubAction<()> =
/// comment_sub("/*", "*/");
/// # let action: SubAction<()> =
/// comment_sub("<!--", "-->");
/// ```
pub fn comment_sub<ActionState>(
  open: impl Into<String>,
  close: impl Into<String>,
) -> SubAction<ActionState> {
  let open: String = open.into();
  let close: String = close.into();

  simple(move |input| {
    // open mismatch
    if !input.rest().starts_with(&open) {
      return 0;
    }

    input.rest()[open.len()..]
      .find(&close)
      // if match, return total length
      .map(|i| i + open.len() + close.len())
      // if the close is not found,
      // accept all rest as the comment
      .unwrap_or(input.rest().len())
  })
}

/// Match from the `open` to the `close`, including the `open` and `close`.
/// If the `close` is not found, accept all the rest.
///
/// The [`Action::head_matcher`] will be set automatically.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{Action, comment};
/// // single line comment
/// # let action: Action<_> =
/// comment("//", "\n");
/// # let action: Action<_> =
/// comment("#", "\n");
/// // multi line comment
/// # let action: Action<_> =
/// comment("/*", "*/");
/// # let action: Action<_> =
/// comment("<!--", "-->");
/// ```
pub fn comment<ActionState: 'static, ErrorType>(
  open: impl Into<String>,
  close: impl Into<String>,
) -> Action<MockTokenKind<()>, ActionState, ErrorType> {
  let open: String = open.into();
  let first = open.chars().next().unwrap();

  Action::from(comment_sub(open, close).into()).unchecked_head_in([first])
}

// TODO: add string & numeric utils

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{ActionInput, HeadMatcher};

  fn assert_accept(action: &Action<MockTokenKind<()>>, text: &str, expected: usize) {
    assert_eq!(
      action
        .exec(&mut ActionInput::new(text, 0, ()).unwrap())
        .unwrap()
        .digested,
      expected
    );
  }
  fn assert_reject(action: &Action<MockTokenKind<()>>, text: &str) {
    assert!(action
      .exec(&mut ActionInput::new(text, 0, ()).unwrap())
      .is_none());
  }

  #[test]
  fn action_utils_comment() {
    let action: Action<MockTokenKind<()>> = comment("//", "\n");

    // common cases
    let text = "// this is a comment\n";
    assert_reject(&action, "123");
    assert_reject(&action, "  // ");
    assert_reject(&action, "/");
    assert_accept(&action, &text, text.len());

    // no close
    let text = "// this is a comment";
    assert_accept(&action, &text, text.len());

    // head matcher
    assert!(matches!(
      action.head_matcher().as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.len() == 1 && set.contains(&'/')
    ));
  }
}
