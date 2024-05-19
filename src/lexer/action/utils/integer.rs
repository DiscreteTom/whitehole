mod accumulator;
mod data;
mod options;

use crate::lexer::{
  action::{simple_with_data, Action, HeadMatcher},
  token::MockTokenKind,
};
use std::collections::HashSet;

pub use accumulator::*;
pub use data::*;
pub use options::*;

/// Try to match an integer literal body in the rest of the input text
/// with the default [`IntegerLiteralBodyOptions`].
/// Return how many bytes are digested and the integer literal data.
pub fn integer_literal_body(
  rest: &str,
  is_body: impl Fn(&char) -> bool,
) -> (usize, IntegerLiteralData<()>) {
  integer_literal_body_with_options(rest, is_body, &IntegerLiteralBodyOptions::default())
}

/// Try to match an integer literal body in the rest of the input text
/// with the given [`IntegerLiteralBodyOptions`].
/// Return how many bytes are digested and the integer literal data.
pub fn integer_literal_body_with<Acc: IntegerLiteralBodyAccumulator>(
  rest: &str,
  is_body: impl Fn(&char) -> bool,
  options_builder: impl FnOnce(
    IntegerLiteralBodyOptions<MockIntegerLiteralBodyAccumulator>,
  ) -> IntegerLiteralBodyOptions<Acc>,
) -> (usize, IntegerLiteralData<Acc::Target>) {
  integer_literal_body_with_options(
    rest,
    is_body,
    &options_builder(IntegerLiteralBodyOptions::default()),
  )
}

/// Try to match an integer literal body in the rest of the input text
/// with the given [`IntegerLiteralBodyOptions`].
/// Return how many bytes are digested and the integer literal data.
pub fn integer_literal_body_with_options<Acc: IntegerLiteralBodyAccumulator>(
  rest: &str,
  is_body: impl Fn(&char) -> bool,
  options: &IntegerLiteralBodyOptions<Acc>,
) -> (usize, IntegerLiteralData<Acc::Target>) {
  let mut separators = vec![];
  let mut digested = 0;

  // TODO: simplify code with macro?
  let body = match (&options.sep, options.acc.clone()) {
    (Some(sep), Some(mut acc)) => {
      for c in rest.chars() {
        if is_body(&c) {
          acc.update(&c);
          digested += c.len_utf8();
        } else if c == *sep {
          separators.push(digested);
          digested += c.len_utf8();
        } else {
          break;
        }
      }
      acc.emit()
    }
    (Some(separator), None) => {
      for c in rest.chars() {
        if is_body(&c) {
          digested += c.len_utf8();
        } else if c == *separator {
          separators.push(digested);
          digested += c.len_utf8();
        } else {
          break;
        }
      }
      Acc::Target::default()
    }
    (None, Some(mut acc)) => {
      for c in rest.chars() {
        if is_body(&c) {
          acc.update(&c);
          digested += c.len_utf8();
        } else {
          break;
        }
      }
      acc.emit()
    }
    (None, None) => {
      for c in rest.chars() {
        if is_body(&c) {
          digested += c.len_utf8();
        } else {
          break;
        }
      }
      Acc::Target::default()
    }
  };

  (digested, IntegerLiteralData { separators, body })
}

macro_rules! generate_integer_literal_functions {
  (
    $body_fn_name:ident,
    $body_fn_name_with:ident,
    $body_fn_name_with_options:ident,
    $action_fn_name:ident,
    $action_fn_name_with:ident,
    $action_fn_name_with_options:ident,
    $prefix:literal,
    $is_body: expr,
    $head_matcher: expr
  ) => {
    /// Try to match the integer literal body in the rest of the input text
    /// with the default [`IntegerLiteralBodyOptions`].
    /// Return how many bytes are digested and the integer literal data.
    pub fn $body_fn_name(rest: &str) -> (usize, IntegerLiteralData<()>) {
      $body_fn_name_with_options(rest, &IntegerLiteralBodyOptions::default())
    }

    /// Try to match the integer literal body in the rest of the input text
    /// with the given [`IntegerLiteralBodyOptions`].
    /// Return how many bytes are digested and the integer literal data.
    pub fn $body_fn_name_with<Acc: IntegerLiteralBodyAccumulator>(
      rest: &str,
      options_builder: impl FnOnce(
        IntegerLiteralBodyOptions<MockIntegerLiteralBodyAccumulator>,
      ) -> IntegerLiteralBodyOptions<Acc>,
    ) -> (usize, IntegerLiteralData<Acc::Target>) {
      $body_fn_name_with_options(rest, &options_builder(IntegerLiteralBodyOptions::default()))
    }

    /// Try to match the integer literal body in the rest of the input text
    /// with the given [`IntegerLiteralBodyOptions`].
    /// Return how many bytes are digested and the integer literal data.
    pub fn $body_fn_name_with_options<Acc: IntegerLiteralBodyAccumulator>(
      rest: &str,
      options: &IntegerLiteralBodyOptions<Acc>,
    ) -> (usize, IntegerLiteralData<Acc::Target>) {
      integer_literal_body_with_options(rest, $is_body, options)
    }

    /// Create an [`Action`] that tries to match the integer literal body
    /// in the rest of the input text
    /// with the default [`IntegerLiteralBodyOptions`].
    pub fn $action_fn_name<ActionState, ErrorType>(
    ) -> Action<MockTokenKind<IntegerLiteralData<()>>, ActionState, ErrorType> {
      $action_fn_name_with_options(IntegerLiteralBodyOptions::default())
    }

    /// Create an [`Action`] that tries to match the integer literal body
    /// in the rest of the input text
    /// with the given [`IntegerLiteralBodyOptions`].
    pub fn $action_fn_name_with<
      ActionState,
      ErrorType,
      Acc: IntegerLiteralBodyAccumulator + 'static,
    >(
      options_builder: impl FnOnce(
        IntegerLiteralBodyOptions<MockIntegerLiteralBodyAccumulator>,
      ) -> IntegerLiteralBodyOptions<Acc>,
    ) -> Action<MockTokenKind<IntegerLiteralData<Acc::Target>>, ActionState, ErrorType> {
      $action_fn_name_with_options(options_builder(IntegerLiteralBodyOptions::default()))
    }

    /// Create an [`Action`] that tries to match the integer literal body
    /// in the rest of the input text
    /// with the given [`IntegerLiteralBodyOptions`].
    pub fn $action_fn_name_with_options<
      ActionState,
      ErrorType,
      Acc: IntegerLiteralBodyAccumulator + 'static,
    >(
      options: IntegerLiteralBodyOptions<Acc>,
    ) -> Action<MockTokenKind<IntegerLiteralData<Acc::Target>>, ActionState, ErrorType> {
      let mut a = simple_with_data(move |input| {
        let prefix = $prefix;
        if input.rest().starts_with(prefix) {
          let (digested, data) =
            $body_fn_name_with_options(&input.rest()[prefix.len()..], &options);
          Some((digested + prefix.len(), data))
        } else {
          None
        }
      });
      a.head_matcher = Some(HeadMatcher::OneOf(HashSet::from($head_matcher)));
      a
    }
  };
}

generate_integer_literal_functions!(
  binary_integer_literal_body,
  binary_integer_literal_body_with,
  binary_integer_literal_body_with_options,
  binary_integer_literal,
  binary_integer_literal_with,
  binary_integer_literal_with_options,
  "0b",
  |c| matches!(c, '0' | '1'),
  ['0']
);

generate_integer_literal_functions!(
  octal_integer_literal_body,
  octal_integer_literal_body_with,
  octal_integer_literal_body_with_options,
  octal_integer_literal,
  octal_integer_literal_with,
  octal_integer_literal_with_options,
  "0o",
  |c| matches!(c, '0'..='7'),
  ['0']
);

generate_integer_literal_functions!(
  decimal_integer_literal_body,
  decimal_integer_literal_body_with,
  decimal_integer_literal_body_with_options,
  decimal_integer_literal,
  decimal_integer_literal_with,
  decimal_integer_literal_with_options,
  "",
  |c| c.is_ascii_digit(),
  ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']
);

generate_integer_literal_functions!(
  hexadecimal_integer_literal_body,
  hexadecimal_integer_literal_body_with,
  hexadecimal_integer_literal_body_with_options,
  hexadecimal_integer_literal,
  hexadecimal_integer_literal_with,
  hexadecimal_integer_literal_with_options,
  "0x",
  |c| c.is_ascii_hexdigit(),
  ['0']
);
