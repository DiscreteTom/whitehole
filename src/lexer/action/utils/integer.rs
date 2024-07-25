mod data;
mod options;

pub use data::*;
pub use options::*;

use crate::{
  lexer::{
    action::{simple_with_data, Action},
    token::MockTokenKind,
  },
  utils::Accumulator,
};

/// Try to match an integer literal body in the rest of the input text
/// with the given [`IntegerLiteralBodyOptions`].
/// E.g. in `0x1_23`, the body is `1_23`, the value is `123`, 4 bytes will be digested.
/// Return how many bytes are digested and the integer literal data.
/// # Caveat
/// If the matched content is separators only, no bytes will be digested
/// (`return.0` will be set to `0`).
/// E.g. if the separator char is `'_'`, then
/// `___` won't be treated as a valid integer literal body.
///
/// However, the matched content may starts/ends with separators.
/// E.g. if the separator char is `'_'`, then
/// `_123`/`123_` will be treated as a valid integer literal body.
/// This is because in some languages (e.g. rust) `0x_123_` is a valid integer literal,
/// however in other languages (e.g. javascript) `0x_123` or `0x123_` is not a valid integer literal.
/// So it's up to the caller to decide whether to accept the leading/trailing separators.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{integer_literal_body_with};
/// let (digested, data) = integer_literal_body_with(
///   "1_234z",
///   |c| c.is_ascii_digit(),
///   |o| o.separator_with(|s| s.ch('_').acc(vec![])).value(String::new())
/// );
/// assert_eq!(digested, 5);
/// assert_eq!(data.separators, vec![1]);
/// assert_eq!(data.value, "1234".to_string());
///
/// // separators only
/// let (digested, data) = integer_literal_body_with(
///   "____z",
///   |c| c.is_ascii_digit(),
///   |o| o.separator_with(|s| s.ch('_').acc(vec![])).value(String::new())
/// );
/// assert_eq!(digested, 0); // digested will be set to 0
/// assert_eq!(data.separators, vec![0, 1, 2, 3]); // separators will still be collected
/// assert_eq!(data.value, "".to_string()); // no value
/// ```
pub fn integer_literal_body_with<
  SepAcc: NumericSeparatorAccumulator,
  ValueAcc: Accumulator<char>,
>(
  rest: &str,
  is_body: impl Fn(char) -> bool,
  options_builder: impl FnOnce(
    IntegerLiteralBodyOptions<(), ()>,
  ) -> IntegerLiteralBodyOptions<SepAcc, ValueAcc>,
) -> (usize, IntegerLiteralData<SepAcc::Acc, ValueAcc>) {
  integer_literal_body_with_options(
    rest,
    is_body,
    options_builder(IntegerLiteralBodyOptions::new()),
  )
}

/// Try to match an integer literal body in the rest of the input text
/// with the given [`IntegerLiteralBodyOptions`].
/// Return how many bytes are digested and the integer literal data.
///
/// E.g. in `0x1_23`, the body is `1_23`, the value is `123`, 4 bytes will be digested.
/// # Caveat
/// If the matched content is separators only, no bytes will be digested
/// (`return.0` will be set to `0`).
/// E.g. if the separator char is `'_'`, then
/// `___` won't be treated as a valid integer literal body.
///
/// However, the matched content may starts/ends with separators.
/// E.g. if the separator char is `'_'`, then
/// `_123`/`123_` will be treated as a valid integer literal body.
/// This is because in some languages (e.g. rust) `0x_123_` is a valid integer literal,
/// however in other languages (e.g. javascript) `0x_123` or `0x123_` is not a valid integer literal.
/// So it's up to the caller to decide whether to accept the leading/trailing separators.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{integer_literal_body_with_options, IntegerLiteralBodyOptions};
/// let (digested, data) = integer_literal_body_with_options(
///   "1_234z",
///   |c| c.is_ascii_digit(),
///   IntegerLiteralBodyOptions::new()
///     .separator_with(|s| s.ch('_').acc(vec![]))
///     .value(String::new())
/// );
/// assert_eq!(digested, 5);
/// assert_eq!(data.separators, vec![1]);
/// assert_eq!(data.value, "1234".to_string());
///
/// // separators only
/// let (digested, data) = integer_literal_body_with_options(
///   "____z",
///   |c| c.is_ascii_digit(),
///   IntegerLiteralBodyOptions::new()
///     .separator_with(|s| s.ch('_').acc(vec![]))
///     .value(String::new())
/// );
/// assert_eq!(digested, 0); // digested will be set to 0
/// assert_eq!(data.separators, vec![0, 1, 2, 3]); // separators will still be collected
/// assert_eq!(data.value, "".to_string()); // no value
/// ```
pub fn integer_literal_body_with_options<
  SepAcc: NumericSeparatorAccumulator,
  ValueAcc: Accumulator<char>,
>(
  rest: &str,
  is_body: impl Fn(char) -> bool,
  mut options: IntegerLiteralBodyOptions<SepAcc, ValueAcc>,
) -> (usize, IntegerLiteralData<SepAcc::Acc, ValueAcc>) {
  let mut digested = 0;
  let mut sep_only = true;

  for c in rest.chars() {
    if options.separator.validate(c) {
      options.separator.update(digested);
      digested += c.len_utf8();
      continue;
    }

    if is_body(c) {
      sep_only = false;
      options.value_to.update(c);
      digested += c.len_utf8();
      continue;
    }

    // not a separator or body char
    break;
  }

  if sep_only {
    digested = 0;
  }

  (
    digested,
    IntegerLiteralData {
      separators: options.separator.emit(),
      value: options.value_to,
    },
  )
}

macro_rules! generate_integer_literal_functions {
  (
    $body_fn_name_with:ident,
    $body_fn_name_with_options:ident,
    $action_fn_name_with:ident,
    $action_fn_name_with_options:ident,
    $prefix:literal,
    $is_body: expr,
    $head: expr
  ) => {
    /// Try to match the integer literal body in the rest of the input text
    /// with the given [`IntegerLiteralBodyOptions`].
    /// Return how many bytes are digested and the integer literal data.
    /// # Caveat
    /// If the matched content is separators only, no bytes will be digested
    /// (the return value will be set to `0`).
    /// E.g. if the separator char is `'_'`, then
    /// `___` won't be treated as a valid integer literal body.
    ///
    /// However, the matched content may starts/ends with separators.
    /// E.g. if the separator char is `'_'`, then
    /// `_123`/`123_` will be treated as a valid integer literal body.
    /// This is because in some languages (e.g. rust) `0x_123_` is a valid integer literal,
    /// however in other languages (e.g. javascript) `0x_123` or `0x123_` is not a valid integer literal.
    /// So it's up to the caller to decide whether to accept the leading/trailing separators.
    pub fn $body_fn_name_with<SepAcc: NumericSeparatorAccumulator, ValueAcc: Accumulator<char>>(
      rest: &str,
      options_builder: impl FnOnce(
        IntegerLiteralBodyOptions<(), ()>,
      ) -> IntegerLiteralBodyOptions<SepAcc, ValueAcc>,
    ) -> (usize, IntegerLiteralData<SepAcc::Acc, ValueAcc>) {
      $body_fn_name_with_options(rest, options_builder(IntegerLiteralBodyOptions::new()))
    }

    /// Try to match the integer literal body in the rest of the input text
    /// with the given [`IntegerLiteralBodyOptions`].
    /// Return how many bytes are digested and the integer literal data.
    /// # Caveat
    /// If the matched content is separators only, no bytes will be digested
    /// (the return value will be set to `0`).
    /// E.g. if the separator char is `'_'`, then
    /// `___` won't be treated as a valid integer literal body.
    ///
    /// However, the matched content may starts/ends with separators.
    /// E.g. if the separator char is `'_'`, then
    /// `_123`/`123_` will be treated as a valid integer literal body.
    /// This is because in some languages (e.g. rust) `0x_123_` is a valid integer literal,
    /// however in other languages (e.g. javascript) `0x_123` or `0x123_` is not a valid integer literal.
    /// So it's up to the caller to decide whether to accept the leading/trailing separators.
    pub fn $body_fn_name_with_options<
      SepAcc: NumericSeparatorAccumulator,
      ValueAcc: Accumulator<char>,
    >(
      rest: &str,
      options: IntegerLiteralBodyOptions<SepAcc, ValueAcc>,
    ) -> (usize, IntegerLiteralData<SepAcc::Acc, ValueAcc>) {
      integer_literal_body_with_options(rest, $is_body, options)
    }

    /// Create an [`Action`] that tries to match the integer literal
    /// in the rest of the input text
    /// with the given [`IntegerLiteralBodyOptions`].
    ///
    /// The [`Action::head`] will be set automatically.
    /// # Caveat
    /// If the integer literal's body is separators only, the action will be rejected.
    /// E.g. if the separator char is `'_'`, then
    /// `___` won't be treated as a valid integer literal body.
    ///
    /// However, the integer literal's body may starts/ends with separators
    /// (decimal integer literal actions will reject if the body starts with a separator).
    /// E.g. if the separator char is `'_'`, then
    /// `_123`/`123_` will be treated as a valid integer literal body.
    /// This is because in some languages (e.g. rust) `0x_123_` is a valid integer literal,
    /// however in other languages (e.g. javascript) `0x_123` or `0x123_` is not a valid integer literal.
    /// So it's up to the caller to decide whether to accept the leading/trailing separators.
    pub fn $action_fn_name_with<
      ActionState,
      ErrorType,
      SepAcc: NumericSeparatorAccumulator + Clone + 'static,
      ValueAcc: Accumulator<char> + Clone + 'static,
    >(
      options_builder: impl FnOnce(
        IntegerLiteralBodyOptions<(), ()>,
      ) -> IntegerLiteralBodyOptions<SepAcc, ValueAcc>,
    ) -> Action<MockTokenKind<IntegerLiteralData<SepAcc::Acc, ValueAcc>>, ActionState, ErrorType> {
      $action_fn_name_with_options(options_builder(IntegerLiteralBodyOptions::new()))
    }

    /// Create an [`Action`] that tries to match the integer literal
    /// in the rest of the input text
    /// with the given [`IntegerLiteralBodyOptions`].
    ///
    /// The [`Action::head`] will be set automatically.
    /// # Caveat
    /// If the integer literal's body is separators only, the action will be rejected.
    /// E.g. if the separator char is `'_'`, then
    /// `___` won't be treated as a valid integer literal body.
    ///
    /// However, the integer literal's body may starts/ends with separators
    /// (decimal integer literal actions will reject if the body starts with a separator).
    /// E.g. if the separator char is `'_'`, then
    /// `_123`/`123_` will be treated as a valid integer literal body.
    /// This is because in some languages (e.g. rust) `0x_123_` is a valid integer literal,
    /// however in other languages (e.g. javascript) `0x_123` or `0x123_` is not a valid integer literal.
    /// So it's up to the caller to decide whether to accept the leading/trailing separators.
    pub fn $action_fn_name_with_options<
      ActionState,
      ErrorType,
      SepAcc: NumericSeparatorAccumulator + Clone + 'static,
      ValueAcc: Accumulator<char> + Clone + 'static,
    >(
      options: IntegerLiteralBodyOptions<SepAcc, ValueAcc>,
    ) -> Action<MockTokenKind<IntegerLiteralData<SepAcc::Acc, ValueAcc>>, ActionState, ErrorType> {
      let prefix = $prefix;

      if prefix.len() == 0 {
        // no prefix, decimal integer literal
        simple_with_data(move |input| {
          let (digested, data) = $body_fn_name_with_options(&input.rest(), options.clone());

          if digested == 0 {
            return None;
          }

          // reject if the first char is a separator
          if options.separator.validate(input.next()) {
            return None;
          }

          Some((digested, data))
        })
      } else {
        simple_with_data(move |input| {
          if !input.rest().starts_with(prefix) {
            return None;
          }
          let (digested, data) =
            $body_fn_name_with_options(&input.rest()[prefix.len()..], options.clone());
          if digested == 0 {
            return None;
          }
          Some((digested + prefix.len(), data))
        })
      }
      .unchecked_head_in($head)
    }
  };
}

generate_integer_literal_functions!(
  binary_integer_literal_body_with,
  binary_integer_literal_body_with_options,
  binary_integer_literal_with,
  binary_integer_literal_with_options,
  "0b",
  |c| matches!(c, '0' | '1'),
  ['0']
);

generate_integer_literal_functions!(
  octal_integer_literal_body_with,
  octal_integer_literal_body_with_options,
  octal_integer_literal_with,
  octal_integer_literal_with_options,
  "0o",
  |c| matches!(c, '0'..='7'),
  ['0']
);

generate_integer_literal_functions!(
  decimal_integer_literal_body_with,
  decimal_integer_literal_body_with_options,
  decimal_integer_literal_with,
  decimal_integer_literal_with_options,
  "",
  |c| c.is_ascii_digit(),
  ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']
);

generate_integer_literal_functions!(
  hexadecimal_integer_literal_body_with,
  hexadecimal_integer_literal_body_with_options,
  hexadecimal_integer_literal_with,
  hexadecimal_integer_literal_with_options,
  "0x",
  |c| c.is_ascii_hexdigit(),
  ['0']
);

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::ActionInput;

  fn assert_integer_literal_body(
    (digested, data): (usize, IntegerLiteralData<Vec<usize>, String>),
    expect_value: &str,
  ) {
    assert_eq!(digested, 4);
    assert_eq!(data.separators, vec![1]);
    assert_eq!(data.value, expect_value.to_string());
  }

  #[test]
  fn test_integer_literal_body_with_options() {
    let options_builder = |o: IntegerLiteralBodyOptions<(), ()>| {
      o.separator_with(|s| s.indexes_to_vec()).value_to_string()
    };
    let options = IntegerLiteralBodyOptions::new()
      .separator_with(|s| s.indexes_to_vec())
      .value_to_string();
    assert_integer_literal_body(
      binary_integer_literal_body_with("1_01", options_builder),
      "101",
    );
    assert_integer_literal_body(
      binary_integer_literal_body_with_options("1_01", options.clone()),
      "101",
    );
    assert_integer_literal_body(
      octal_integer_literal_body_with("7_07", options_builder),
      "707",
    );
    assert_integer_literal_body(
      octal_integer_literal_body_with_options("7_07", options.clone()),
      "707",
    );
    assert_integer_literal_body(
      decimal_integer_literal_body_with("9_09", options_builder),
      "909",
    );
    assert_integer_literal_body(
      decimal_integer_literal_body_with_options("9_09", options.clone()),
      "909",
    );
    assert_integer_literal_body(
      hexadecimal_integer_literal_body_with("f_0f", options_builder),
      "f0f",
    );
    assert_integer_literal_body(
      hexadecimal_integer_literal_body_with_options("f_0f", options.clone()),
      "f0f",
    );
  }

  fn assert_default_integer_literal_action(
    action: Action<MockTokenKind<IntegerLiteralData<(), ()>>>,
    s: &str,
    expect_digested: usize,
  ) {
    let res = action
      .exec(&mut ActionInput::new(s, 0, &mut ()).unwrap())
      .unwrap();
    assert_eq!(res.digested, expect_digested);
  }

  fn assert_reject(action: Action<MockTokenKind<IntegerLiteralData<(), ()>>>, s: &str) {
    assert!(action
      .exec(&mut ActionInput::new(s, 0, &mut ()).unwrap())
      .is_none());
  }

  fn assert_integer_literal_action(
    action: Action<MockTokenKind<IntegerLiteralData<Vec<usize>, String>>>,
    s: &str,
    expect_value: &str,
  ) {
    let res = action
      .exec(&mut ActionInput::new(s, 0, &mut ()).unwrap())
      .unwrap();
    assert_eq!(res.digested, 6);
    assert_eq!(res.kind.data.separators, vec![1]);
    assert_eq!(res.kind.data.value, expect_value);
  }

  #[test]
  fn test_integer_literal_actions() {
    let options_builder = |o: IntegerLiteralBodyOptions<(), ()>| {
      o.separator_with(|s| s.indexes_to_vec()).value_to_string()
    };
    let options = IntegerLiteralBodyOptions::new()
      .separator_with(|s| s.indexes_to_vec())
      .value_to_string();

    assert_integer_literal_action(
      binary_integer_literal_with(options_builder),
      "0b1_01z",
      "101",
    );
    assert_integer_literal_action(
      binary_integer_literal_with_options(options.clone()),
      "0b1_01z",
      "101",
    );
    assert_integer_literal_action(
      octal_integer_literal_with(options_builder),
      "0o7_07z",
      "707",
    );
    assert_integer_literal_action(
      octal_integer_literal_with_options(options.clone()),
      "0o7_07z",
      "707",
    );
    assert_integer_literal_action(
      decimal_integer_literal_with(options_builder),
      "9_0909z",
      "90909",
    );
    assert_integer_literal_action(
      decimal_integer_literal_with_options(options.clone()),
      "9_0909z",
      "90909",
    );
    assert_integer_literal_action(
      hexadecimal_integer_literal_with(options_builder),
      "0xf_0fz",
      "f0f",
    );
    assert_integer_literal_action(
      hexadecimal_integer_literal_with_options(options.clone()),
      "0xf_0fz",
      "f0f",
    );
  }

  #[test]
  fn with_suffix() {
    // this is an example of customize an action with literal body utils
    let action_factory = || {
      simple_with_data(|input| {
        let prefix = "0B";
        let suffix = "n"; // just like the big int literal in js/ts

        // check prefix
        if input.rest().starts_with(prefix) {
          // eat body
          let (digested, data) =
            binary_integer_literal_body_with(&input.rest()[prefix.len()..], |o| o);
          // check suffix
          if input.rest()[prefix.len() + digested..].starts_with(suffix) {
            Some((prefix.len() + digested + suffix.len(), data))
          } else {
            None
          }
        } else {
          None
        }
      })
      .unchecked_head_in(['0'])
    };
    assert_default_integer_literal_action(action_factory(), "0B101n", 6);
    // missing suffix
    assert_reject(action_factory(), "0B101");
    // wrong prefix
    assert_reject(action_factory(), "0b101n");
  }
}
