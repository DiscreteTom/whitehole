//! ## For Developers
//!
//! Here is the recommended order of reading the source code:
//!
//! 1. [`self::body`]
//! 2. [`self::options`]
//! 3. [`self::escape`]
//! 4. [`self`]

mod body;
mod error;
mod escape;
mod options;
mod value;

use std::collections::HashSet;

pub use body::*;
pub use error::*;
pub use escape::*;
pub use options::*;
pub use value::*;

use super::{Accumulator, StringList};
use crate::lexer::{
  action::{simple_with_data, Action},
  token::MockTokenKind,
};

// TODO: comments
pub fn string<
  ActionState,
  ErrorType,
  Value: PartialStringBodyValue + 'static,
  CustomError: 'static,
  BodyAcc: Accumulator<PartialStringBody<Value, CustomError>> + Clone,
>(
  open: impl Into<StringList>,
  options: StringBodyOptions<Value, CustomError, BodyAcc>,
) -> Action<MockTokenKind<BodyAcc>, ActionState, ErrorType> {
  let open: Vec<String> = open.into().0;
  let head: HashSet<_> = open
    .iter()
    .map(|s| {
      s.chars()
        .next()
        .expect("string literal's open quote should not be empty")
    })
    .collect();

  simple_with_data(move |input| {
    for prefix in &open {
      if input.rest().starts_with(prefix) {
        let (body_len, data) = string_body(&input.rest()[prefix.len()..], &options);
        return Some((prefix.len() + body_len, data));
      }
    }
    // no prefix matched
    return None;
  })
  .unchecked_head_in(head)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{ActionInput, ActionOutput, HeadMatcher};

  fn exec_action(
    action: &Action<MockTokenKind<Vec<PartialStringBody<String, ()>>>, (), ()>,
    text: &str,
  ) -> Option<ActionOutput<MockTokenKind<Vec<PartialStringBody<String, ()>>>, Option<()>>> {
    action.exec(&mut ActionInput::new(text, 0, &mut ()).unwrap())
  }

  fn validate_output(
    output: ActionOutput<MockTokenKind<Vec<PartialStringBody<String, ()>>>, Option<()>>,
  ) -> ActionOutput<MockTokenKind<Vec<PartialStringBody<String, ()>>>, Option<()>> {
    // ensure at least one partial string body (the unterminated error)
    assert!(output.kind.data.len() > 0);

    // ensure only the last partial string body is the close
    output.kind.data.iter().enumerate().for_each(|(i, p)| {
      assert!(if i == output.kind.data.len() - 1 {
        p.close
      } else {
        !p.close
      });
    });

    output
  }

  fn assert_accept_all(
    action: &Action<MockTokenKind<Vec<PartialStringBody<String, ()>>>, (), ()>,
    text: &str,
    value: &str,
  ) -> ActionOutput<MockTokenKind<Vec<PartialStringBody<String, ()>>>, Option<()>> {
    let output = exec_action(action, text).unwrap();
    assert_eq!(output.digested, text.len());
    assert_eq!(
      output
        .kind
        .data
        .iter()
        .map(|p| p.value.clone())
        .collect::<Vec<_>>()
        .join(""),
      value
    );
    validate_output(output)
  }

  #[test]
  fn test_string_literal() {
    let a = string(
      [
        "\"",  // normal double quote string
        "c\"", // c string
        "r\"", // raw string
      ],
      StringBodyOptions::default()
        .close('"')
        .singleline()
        .escape(
          '\\',
          [
            map([('n', '\n'), ('t', '\t')]),
            line_continuation(["\r\n", "\n"]),
            code_point_with(|o| o.error(|_| ())),
            hex_with(|o| o.error(|_| ())),
            unicode_with(|o| o.error(|_| ())),
          ],
        )
        .chars(|c| c.is_ascii_digit())
        .acc_to_vec(),
    );

    // head matcher
    assert!(
      matches!(&a.head_matcher, Some(HeadMatcher::OneOf(set)) if set == &HashSet::from(['"', 'c', 'r']))
    );

    // wrong prefix
    assert!(exec_action(&a, "aa").is_none());

    // unterminated
    let output = assert_accept_all(&a, "\"", "");
    assert_eq!(output.kind.data.len(), 1);
    assert!(matches!(
      output.kind.data[0].error,
      Some(StringLiteralError::Unterminated)
    ));

    // no matcher matches
    let output = validate_output(exec_action(&a, "\"a").unwrap());
    assert_eq!(output.digested, 1);
    assert_eq!(output.kind.data.len(), 1);
    assert_eq!(output.kind.data[0].value, "");
    assert!(matches!(
      output.kind.data[0].error,
      Some(StringLiteralError::Unterminated)
    ));

    // terminate on line break
    let output = validate_output(exec_action(&a, "\"\n").unwrap());
    assert_eq!(output.digested, 1); // new line is not digested
    assert_eq!(output.kind.data.len(), 1);
    assert_eq!(output.kind.data[0].value, "");
    assert!(matches!(
      output.kind.data[0].error,
      Some(StringLiteralError::Unterminated)
    ));

    // unhandled escape
    let output = assert_accept_all(&a, "\"\\1\"", "\\1");
    assert_eq!(output.kind.data.len(), 3);
    assert!(matches!(
      output.kind.data[0].error,
      Some(StringLiteralError::UnhandledEscape)
    ));
    assert!(matches!(output.kind.data[1].error, None));

    // all together
    let value = "123\n\t\x11\u{1234}\u{1234}\\1";
    let text = "\"123\\n\\t\\\r\n\\\n\\x11\\u1234\\u{1234}\\1\"";
    assert_accept_all(&a, text, value);
    assert_accept_all(&a, &(String::from("c") + text), value);
    assert_accept_all(&a, &(String::from("r") + text), value);
  }
}
