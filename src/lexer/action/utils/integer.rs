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
/// If the matched content is separators only, no bytes will be digested,
/// the first element of the return tuple will be set to `0`.
/// E.g. if the separator char is `'_'`, then
/// `___` won't be treated as a valid integer literal body.
///
/// However, the matched content may starts or ends with separators.
/// E.g. if the separator char is `'_'`, then
/// `_123` and `123_` will be treated as a valid integer literal body.
/// This is because in some languages (e.g. rust) `0x_123_` is a valid integer literal,
/// however in other languages (e.g. javascript) `0x_123` and `0x123_` are not valid integer literals.
/// So it's up to the caller to decide whether to accept the leading/trailing separators.
///
/// Similarly, consecutive separators are allowed in a valid integer literal body.
/// It's up to the caller to decide whether to accept consecutive separators.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{integer_literal_body_with};
/// let (digested, data) = integer_literal_body_with(
///   "1_234z",
///   |c| c.is_ascii_digit(),
///   |o| o
///     .separator_with(|s| s.indexes_to_vec())
///     .value_to_string()
/// );
/// assert_eq!(digested, 5); // `1_234`
/// assert_eq!(data.separators, vec![1]);
/// assert_eq!(data.value, "1234".to_string());
///
/// // separators only
/// let (digested, data) = integer_literal_body_with(
///   "____z",
///   |c| c.is_ascii_digit(),
///   |o| o
///     .separator_with(|s| s.indexes_to_vec())
///     .value_to_string()
/// );
/// assert_eq!(digested, 0); // digested will be set to 0
/// assert_eq!(data.separators, vec![0, 1, 2, 3]); // separators will still be collected
/// assert_eq!(data.value, "".to_string()); // no value
///
/// // starts or ends with separators
/// let (digested, data) = integer_literal_body_with(
///   "_123_z",
///   |c| c.is_ascii_digit(),
///   |o| o
///     .separator_with(|s| s.indexes_to_vec())
///     .value_to_string()
/// );
/// assert_eq!(digested, 5); // `_123_`
/// assert_eq!(data.separators, vec![0, 4]);
/// assert_eq!(data.value, "123".to_string());
///
/// // consecutive separators
/// let (digested, data) = integer_literal_body_with(
///   "1__2z",
///   |c| c.is_ascii_digit(),
///   |o| o
///     .separator_with(|s| s.indexes_to_vec())
///     .value_to_string()
/// );
/// assert_eq!(digested, 4); // `1__2`
/// assert_eq!(data.separators, vec![1, 2]);
/// assert_eq!(data.value, "12".to_string());
/// ```
#[inline]
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
/// If the matched content is separators only, no bytes will be digested,
/// the first element of the return tuple will be set to `0`.
/// E.g. if the separator char is `'_'`, then
/// `___` won't be treated as a valid integer literal body.
///
/// However, the matched content may starts or ends with separators.
/// E.g. if the separator char is `'_'`, then
/// `_123` and `123_` will be treated as a valid integer literal body.
/// This is because in some languages (e.g. rust) `0x_123_` is a valid integer literal,
/// however in other languages (e.g. javascript) `0x_123` and `0x123_` are not valid integer literals.
/// So it's up to the caller to decide whether to accept the leading/trailing separators.
///
/// Similarly, consecutive separators are allowed in a valid integer literal body.
/// It's up to the caller to decide whether to accept consecutive separators.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{integer_literal_body_with_options, IntegerLiteralBodyOptions};
/// let (digested, data) = integer_literal_body_with_options(
///   "1_234z",
///   |c| c.is_ascii_digit(),
///   IntegerLiteralBodyOptions::new()
///     .separator_with(|s| s.indexes_to_vec())
///     .value_to_string()
/// );
/// assert_eq!(digested, 5); // `1_234`
/// assert_eq!(data.separators, vec![1]);
/// assert_eq!(data.value, "1234".to_string());
///
/// // separators only
/// let (digested, data) = integer_literal_body_with_options(
///   "____z",
///   |c| c.is_ascii_digit(),
///   IntegerLiteralBodyOptions::new()
///     .separator_with(|s| s.indexes_to_vec())
///     .value_to_string()
/// );
/// assert_eq!(digested, 0); // digested will be set to 0
/// assert_eq!(data.separators, vec![0, 1, 2, 3]); // separators will still be collected
/// assert_eq!(data.value, "".to_string()); // no value
///
/// // starts or ends with separators
/// let (digested, data) = integer_literal_body_with_options(
///   "_123_z",
///   |c| c.is_ascii_digit(),
///   IntegerLiteralBodyOptions::new()
///     .separator_with(|s| s.indexes_to_vec())
///     .value_to_string()
/// );
/// assert_eq!(digested, 5); // `_123_`
/// assert_eq!(data.separators, vec![0, 4]);
/// assert_eq!(data.value, "123".to_string());
///
/// // consecutive separators
/// let (digested, data) = integer_literal_body_with_options(
///   "1__2z",
///   |c| c.is_ascii_digit(),
///   IntegerLiteralBodyOptions::new()
///     .separator_with(|s| s.indexes_to_vec())
///     .value_to_string()
/// );
/// assert_eq!(digested, 4); // `1__2`
/// assert_eq!(data.separators, vec![1, 2]);
/// assert_eq!(data.value, "12".to_string());
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
      options.separator.update(digested); // record the separator index
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

  // if the matched content is separators only, no bytes will be digested
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
    /// E.g. in `0x1_23`, the body is `1_23`, the value is `123`, 4 bytes will be digested.
    /// Return how many bytes are digested and the integer literal data.
    /// # Caveat
    /// If the matched content is separators only, no bytes will be digested,
    /// the first element of the return tuple will be set to `0`.
    /// E.g. if the separator char is `'_'`, then
    /// `___` won't be treated as a valid integer literal body.
    ///
    /// However, the matched content may starts or ends with separators.
    /// E.g. if the separator char is `'_'`, then
    /// `_123` and `123_` will be treated as a valid integer literal body.
    /// This is because in some languages (e.g. rust) `0x_123_` is a valid integer literal,
    /// however in other languages (e.g. javascript) `0x_123` and `0x123_` are not valid integer literals.
    /// So it's up to the caller to decide whether to accept the leading/trailing separators.
    ///
    /// Similarly, consecutive separators are allowed in a valid integer literal body.
    /// It's up to the caller to decide whether to accept consecutive separators.
    #[inline]
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
    /// E.g. in `0x1_23`, the body is `1_23`, the value is `123`, 4 bytes will be digested.
    /// Return how many bytes are digested and the integer literal data.
    /// # Caveat
    /// If the matched content is separators only, no bytes will be digested,
    /// the first element of the return tuple will be set to `0`.
    /// E.g. if the separator char is `'_'`, then
    /// `___` won't be treated as a valid integer literal body.
    ///
    /// However, the matched content may starts or ends with separators.
    /// E.g. if the separator char is `'_'`, then
    /// `_123` and `123_` will be treated as a valid integer literal body.
    /// This is because in some languages (e.g. rust) `0x_123_` is a valid integer literal,
    /// however in other languages (e.g. javascript) `0x_123` and `0x123_` are not valid integer literals.
    /// So it's up to the caller to decide whether to accept the leading/trailing separators.
    ///
    /// Similarly, consecutive separators are allowed in a valid integer literal body.
    /// It's up to the caller to decide whether to accept consecutive separators.
    #[inline]
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
    /// However, the matched content may starts or ends with separators.
    /// E.g. if the separator char is `'_'`, then
    /// `_123` and `123_` will be treated as a valid integer literal body.
    /// This is because in some languages (e.g. rust) `0x_123_` is a valid integer literal,
    /// however in other languages (e.g. javascript) `0x_123` and `0x123_` are not valid integer literals.
    /// So it's up to the caller to decide whether to accept the leading/trailing separators.
    ///
    /// Similarly, consecutive separators are allowed in a valid integer literal body.
    /// It's up to the caller to decide whether to accept consecutive separators.
    ///
    /// For decimal integer literals, if the first char is a separator, the action will be rejected.
    #[inline]
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
    /// However, the matched content may starts or ends with separators.
    /// E.g. if the separator char is `'_'`, then
    /// `_123` and `123_` will be treated as a valid integer literal body.
    /// This is because in some languages (e.g. rust) `0x_123_` is a valid integer literal,
    /// however in other languages (e.g. javascript) `0x_123` and `0x123_` are not valid integer literals.
    /// So it's up to the caller to decide whether to accept the leading/trailing separators.
    ///
    /// Similarly, consecutive separators are allowed in a valid integer literal body.
    /// It's up to the caller to decide whether to accept consecutive separators.
    ///
    /// For decimal integer literals, if the first char is a separator, the action will be rejected.
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
          // reject if the first char is a separator
          if options.separator.validate(input.next()) {
            return None;
          }

          let (digested, data) = $body_fn_name_with_options(&input.rest(), options.clone());

          if digested == 0 {
            return None;
          }

          Some((digested, data))
        })
      } else {
        simple_with_data(move |input| {
          // check prefix
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
  "", // no prefix
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

  #[test]
  fn test_integer_literal_body_with() {
    let (digested, data) = integer_literal_body_with(
      "1_234z",
      |c| c.is_ascii_digit(),
      |o| o.separator_with(|s| s.indexes_to_vec()).value_to_string(),
    );
    assert_eq!(digested, 5);
    assert_eq!(data.separators, vec![1]);
    assert_eq!(data.value, "1234".to_string());
  }

  #[test]
  fn test_integer_literal_body_with_options() {
    fn assert_integer_literal_body(
      (digested, data): (usize, IntegerLiteralData<Vec<usize>, String>),
      expect_value: &str,
    ) {
      assert_eq!(digested, 4);
      assert_eq!(data.separators, vec![1]);
      assert_eq!(data.value, expect_value.to_string());
    }

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

  #[test]
  fn test_integer_literal_body_with_options_separators_only() {
    fn test(
      f: impl Fn(&str, IntegerLiteralBodyOptions<(), ()>) -> (usize, IntegerLiteralData<(), ()>),
    ) {
      let options = IntegerLiteralBodyOptions::new();
      assert_eq!(f("____", options).0, 0);
    }
    test(binary_integer_literal_body_with_options);
    test(octal_integer_literal_body_with_options);
    test(decimal_integer_literal_body_with_options);
    test(hexadecimal_integer_literal_body_with_options);
  }

  #[test]
  fn test_integer_literal_body_with_options_starts_or_ends_with_separators() {
    fn test(
      f: impl Fn(
        &str,
        IntegerLiteralBodyOptions<NumericSeparatorOptions<()>, ()>,
      ) -> (usize, IntegerLiteralData<(), ()>),
    ) {
      let options = IntegerLiteralBodyOptions::new().default_separator();
      assert_eq!(f("_000_", options.clone()).0, 5);
      assert_eq!(f("000_", options.clone()).0, 4);
      assert_eq!(f("_000", options).0, 4);
    }
    test(binary_integer_literal_body_with_options);
    test(octal_integer_literal_body_with_options);
    test(decimal_integer_literal_body_with_options);
    test(hexadecimal_integer_literal_body_with_options);
  }

  #[test]
  fn test_integer_literal_body_with_options_consecutive_separators() {
    fn test(
      f: impl Fn(
        &str,
        IntegerLiteralBodyOptions<NumericSeparatorOptions<()>, ()>,
      ) -> (usize, IntegerLiteralData<(), ()>),
    ) {
      let options = IntegerLiteralBodyOptions::new().default_separator();
      assert_eq!(f("0__0", options).0, 4);
    }
    test(binary_integer_literal_body_with_options);
    test(octal_integer_literal_body_with_options);
    test(decimal_integer_literal_body_with_options);
    test(hexadecimal_integer_literal_body_with_options);
  }

  fn assert_reject(action: Action<MockTokenKind<IntegerLiteralData<(), ()>>>, s: &str) {
    assert!(action.exec.as_immutable()(&ActionInput::new(s, 0, &()).unwrap()).is_none());
  }

  #[test]
  fn test_integer_literal_actions() {
    fn assert_integer_literal_action(
      action: Action<MockTokenKind<IntegerLiteralData<Vec<usize>, String>>>,
      s: &str,
      expect_value: &str,
    ) {
      let res = action.exec.as_immutable()(&ActionInput::new(s, 0, &()).unwrap()).unwrap();
      assert_eq!(res.digested, 6);
      assert_eq!(res.kind.data.separators, vec![1]);
      assert_eq!(res.kind.data.value, expect_value);
    }
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
  fn test_decimal_integer_literal_action_starts_with_separator() {
    let action = decimal_integer_literal_with_options(IntegerLiteralBodyOptions::new());
    assert_reject(action, "_123");
  }

  #[test]
  fn test_integer_literal_actions_wrong_prefix() {
    let action = binary_integer_literal_with_options(IntegerLiteralBodyOptions::new());
    assert_reject(action, "z");
    let action = octal_integer_literal_with_options(IntegerLiteralBodyOptions::new());
    assert_reject(action, "z");
    let action = hexadecimal_integer_literal_with_options(IntegerLiteralBodyOptions::new());
    assert_reject(action, "z");
    let action =
      decimal_integer_literal_with_options(IntegerLiteralBodyOptions::new().default_separator());
    assert_reject(action, "_123");
  }

  #[test]
  fn test_no_digested() {
    let action = binary_integer_literal_with_options(IntegerLiteralBodyOptions::new());
    assert_reject(action, "0b");
    let action = octal_integer_literal_with_options(IntegerLiteralBodyOptions::new());
    assert_reject(action, "0o");
    let action = decimal_integer_literal_with_options(IntegerLiteralBodyOptions::new());
    assert_reject(action, "z");
    let action = hexadecimal_integer_literal_with_options(IntegerLiteralBodyOptions::new());
    assert_reject(action, "0x");
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
    assert_eq!(
      action_factory().exec.as_immutable()(&ActionInput::new("0B101n", 0, &()).unwrap())
        .unwrap()
        .digested,
      6
    );
    // missing suffix
    assert_reject(action_factory(), "0B101");
    // wrong prefix
    assert_reject(action_factory(), "0b101n");
  }
}
