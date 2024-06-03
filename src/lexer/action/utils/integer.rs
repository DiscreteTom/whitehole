mod data;
mod options;

// TODO: organize export
pub use data::*;
pub use options::*;

use super::{Accumulator, MockAccumulator};
use crate::lexer::{
  action::{simple_with_data, Action, HeadMatcher},
  token::MockTokenKind,
};
use std::collections::HashSet;

/// Try to match an integer literal body in the rest of the input text
/// with the default [`IntegerLiteralBodyOptions`].
/// E.g. in `0x123`, the body is `123`.
/// Return how many bytes are digested and the integer literal data.
/// # Examples
/// ```
/// # use whitehole::lexer::action::integer_literal_body;
/// let (digested, _) = integer_literal_body("123", |c| c.is_ascii_digit());
/// assert_eq!(digested, 3);
/// ```
pub fn integer_literal_body(
  rest: &str,
  is_body: impl Fn(&char) -> bool,
) -> (usize, IntegerLiteralData<(), ()>) {
  integer_literal_body_with_options(rest, is_body, &IntegerLiteralBodyOptions::default())
}

/// Try to match an integer literal body in the rest of the input text
/// with the given [`IntegerLiteralBodyOptions`].
/// E.g. in `0x123`, the body is `123`.
/// Return how many bytes are digested and the integer literal data.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{integer_literal_body_with};
/// let (digested, data) = integer_literal_body_with(
///   "1_234",
///   |c| c.is_ascii_digit(),
///   |o| o.separator_with(|s| s.acc_to_vec()).value_to_string()
/// );
/// assert_eq!(digested, 5);
/// assert_eq!(data.separators, vec![1]);
/// assert_eq!(data.value, "1234".to_string());
/// ```
pub fn integer_literal_body_with<SepAcc: Accumulator<usize>, ValueAcc: Accumulator<char>>(
  rest: &str,
  is_body: impl Fn(&char) -> bool,
  options_builder: impl FnOnce(
    IntegerLiteralBodyOptions<MockAccumulator, MockAccumulator>,
  ) -> IntegerLiteralBodyOptions<SepAcc, ValueAcc>,
) -> (usize, IntegerLiteralData<SepAcc::Target, ValueAcc::Target>) {
  integer_literal_body_with_options(
    rest,
    is_body,
    &options_builder(IntegerLiteralBodyOptions::default()),
  )
}

/// Try to match an integer literal body in the rest of the input text
/// with the given [`IntegerLiteralBodyOptions`].
/// E.g. in `0x123`, the body is `123`.
/// Return how many bytes are digested and the integer literal data.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{integer_literal_body_with_options, IntegerLiteralBodyOptions};
/// let (digested, data) = integer_literal_body_with_options(
///   "1_234",
///   |c| c.is_ascii_digit(),
///   &IntegerLiteralBodyOptions::default()
///     .separator_with(|s| s.acc_to_vec())
///     .value_to_string()
/// );
/// assert_eq!(digested, 5);
/// assert_eq!(data.separators, vec![1]);
/// assert_eq!(data.value, "1234".to_string());
/// ```
pub fn integer_literal_body_with_options<
  SepAcc: Accumulator<usize>,
  ValueAcc: Accumulator<char>,
>(
  rest: &str,
  is_body: impl Fn(&char) -> bool,
  options: &IntegerLiteralBodyOptions<SepAcc, ValueAcc>,
) -> (usize, IntegerLiteralData<SepAcc::Target, ValueAcc::Target>) {
  let mut digested = 0;

  macro_rules! check_sep {
    ($c:expr, $sep:expr) => {
      if $c == $sep.ch {
        $sep.acc.update(&digested);
        digested += $c.len_utf8();
        continue;
      }
    };
  }
  macro_rules! proc_body {
    ($c:expr) => {
      if is_body(&$c) {
        digested += $c.len_utf8();
        continue;
      }
    };
  }
  macro_rules! proc_body_acc {
    ($c:expr, $acc:expr) => {
      if is_body(&$c) {
        $acc.update(&$c);
        digested += $c.len_utf8();
        continue;
      }
    };
  }

  // TODO: simplify code with macro?
  // check `None` outside the loop to optimize the performance
  let data = match (options.separator.clone(), options.value.clone()) {
    (Some(mut sep), Some(mut acc)) => {
      for c in rest.chars() {
        check_sep!(c, sep);
        proc_body_acc!(c, acc);
        break;
      }
      IntegerLiteralData {
        separators: sep.acc.emit(),
        value: acc.emit(),
      }
    }
    (Some(mut sep), None) => {
      for c in rest.chars() {
        check_sep!(c, sep);
        proc_body!(c);
        break;
      }
      IntegerLiteralData {
        separators: sep.acc.emit(),
        value: ValueAcc::Target::default(),
      }
    }
    (None, Some(mut acc)) => {
      for c in rest.chars() {
        proc_body_acc!(c, acc);
        break;
      }
      IntegerLiteralData {
        separators: SepAcc::Target::default(),
        value: acc.emit(),
      }
    }
    (None, None) => {
      for c in rest.chars() {
        proc_body!(c);
        break;
      }
      IntegerLiteralData {
        separators: SepAcc::Target::default(),
        value: ValueAcc::Target::default(),
      }
    }
  };

  (digested, data)
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
    pub fn $body_fn_name(rest: &str) -> (usize, IntegerLiteralData<(), ()>) {
      $body_fn_name_with_options(rest, &IntegerLiteralBodyOptions::default())
    }

    /// Try to match the integer literal body in the rest of the input text
    /// with the given [`IntegerLiteralBodyOptions`].
    /// Return how many bytes are digested and the integer literal data.
    pub fn $body_fn_name_with<SepAcc: Accumulator<usize>, ValueAcc: Accumulator<char>>(
      rest: &str,
      options_builder: impl FnOnce(
        IntegerLiteralBodyOptions<MockAccumulator, MockAccumulator>,
      ) -> IntegerLiteralBodyOptions<SepAcc, ValueAcc>,
    ) -> (usize, IntegerLiteralData<SepAcc::Target, ValueAcc::Target>) {
      $body_fn_name_with_options(rest, &options_builder(IntegerLiteralBodyOptions::default()))
    }

    /// Try to match the integer literal body in the rest of the input text
    /// with the given [`IntegerLiteralBodyOptions`].
    /// Return how many bytes are digested and the integer literal data.
    pub fn $body_fn_name_with_options<SepAcc: Accumulator<usize>, ValueAcc: Accumulator<char>>(
      rest: &str,
      options: &IntegerLiteralBodyOptions<SepAcc, ValueAcc>,
    ) -> (usize, IntegerLiteralData<SepAcc::Target, ValueAcc::Target>) {
      integer_literal_body_with_options(rest, $is_body, options)
    }

    /// Create an [`Action`] that tries to match the integer literal
    /// in the rest of the input text
    /// with the default [`IntegerLiteralBodyOptions`].
    pub fn $action_fn_name<ActionState, ErrorType>(
    ) -> Action<MockTokenKind<IntegerLiteralData<(), ()>>, ActionState, ErrorType> {
      $action_fn_name_with_options(IntegerLiteralBodyOptions::default())
    }

    /// Create an [`Action`] that tries to match the integer literal
    /// in the rest of the input text
    /// with the given [`IntegerLiteralBodyOptions`].
    pub fn $action_fn_name_with<
      ActionState,
      ErrorType,
      SepAcc: Accumulator<usize> + 'static,
      ValueAcc: Accumulator<char> + 'static,
    >(
      options_builder: impl FnOnce(
        IntegerLiteralBodyOptions<MockAccumulator, MockAccumulator>,
      ) -> IntegerLiteralBodyOptions<SepAcc, ValueAcc>,
    ) -> Action<
      MockTokenKind<IntegerLiteralData<SepAcc::Target, ValueAcc::Target>>,
      ActionState,
      ErrorType,
    > {
      $action_fn_name_with_options(options_builder(IntegerLiteralBodyOptions::default()))
    }

    /// Create an [`Action`] that tries to match the integer literal
    /// in the rest of the input text
    /// with the given [`IntegerLiteralBodyOptions`].
    pub fn $action_fn_name_with_options<
      ActionState,
      ErrorType,
      SepAcc: Accumulator<usize> + 'static,
      ValueAcc: Accumulator<char> + 'static,
    >(
      options: IntegerLiteralBodyOptions<SepAcc, ValueAcc>,
    ) -> Action<
      MockTokenKind<IntegerLiteralData<SepAcc::Target, ValueAcc::Target>>,
      ActionState,
      ErrorType,
    > {
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::ActionInput;

  fn assert_default_integer_literal_body(
    (digested, data): (usize, IntegerLiteralData<(), ()>),
    expect_digested: usize,
  ) {
    assert_eq!(digested, expect_digested);
    assert_eq!(data.separators, ());
    assert_eq!(data.value, ());
  }

  #[test]
  fn test_default_integer_literal_body() {
    assert_default_integer_literal_body(binary_integer_literal_body("zzz"), 0);
    assert_default_integer_literal_body(binary_integer_literal_body("101"), 3);
    assert_default_integer_literal_body(binary_integer_literal_body("123"), 1);
    assert_default_integer_literal_body(octal_integer_literal_body("zzz"), 0);
    assert_default_integer_literal_body(octal_integer_literal_body("707"), 3);
    assert_default_integer_literal_body(octal_integer_literal_body("789"), 1);
    assert_default_integer_literal_body(decimal_integer_literal_body("zzz"), 0);
    assert_default_integer_literal_body(decimal_integer_literal_body("909"), 3);
    assert_default_integer_literal_body(decimal_integer_literal_body("9ab"), 1);
    assert_default_integer_literal_body(hexadecimal_integer_literal_body("zzz"), 0);
    assert_default_integer_literal_body(hexadecimal_integer_literal_body("f0f"), 3);
    assert_default_integer_literal_body(hexadecimal_integer_literal_body("F0F"), 3);
    assert_default_integer_literal_body(hexadecimal_integer_literal_body("fgh"), 1);
  }

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
    let options_builder = |o: IntegerLiteralBodyOptions<MockAccumulator, MockAccumulator>| {
      o.separator_with(|s| s.acc_to_vec()).value_to_string()
    };
    let options = IntegerLiteralBodyOptions::default()
      .separator_with(|s| s.acc_to_vec())
      .value_to_string();
    assert_integer_literal_body(
      binary_integer_literal_body_with("1_01", &options_builder),
      "101",
    );
    assert_integer_literal_body(
      binary_integer_literal_body_with_options("1_01", &options),
      "101",
    );
    assert_integer_literal_body(
      octal_integer_literal_body_with("7_07", &options_builder),
      "707",
    );
    assert_integer_literal_body(
      octal_integer_literal_body_with_options("7_07", &options),
      "707",
    );
    assert_integer_literal_body(
      decimal_integer_literal_body_with("9_09", &options_builder),
      "909",
    );
    assert_integer_literal_body(
      decimal_integer_literal_body_with_options("9_09", &options),
      "909",
    );
    assert_integer_literal_body(
      hexadecimal_integer_literal_body_with("f_0f", &options_builder),
      "f0f",
    );
    assert_integer_literal_body(
      hexadecimal_integer_literal_body_with_options("f_0f", &options),
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

  #[test]
  fn test_default_integer_literal_actions() {
    // wrong prefix
    assert_reject(binary_integer_literal(), "000");
    assert_reject(octal_integer_literal(), "000");
    assert_reject(hexadecimal_integer_literal(), "000");

    // prefix only
    assert_default_integer_literal_action(binary_integer_literal(), "0b", 2);
    assert_default_integer_literal_action(octal_integer_literal(), "0o", 2);
    assert_default_integer_literal_action(hexadecimal_integer_literal(), "0x", 2);

    // prefix + invalid char
    assert_default_integer_literal_action(binary_integer_literal(), "0bz", 2);
    assert_default_integer_literal_action(octal_integer_literal(), "0oz", 2);
    assert_default_integer_literal_action(hexadecimal_integer_literal(), "0xz", 2);

    // valid content
    assert_default_integer_literal_action(binary_integer_literal(), "0b101z", 5);
    assert_default_integer_literal_action(octal_integer_literal(), "0o707z", 5);
    assert_default_integer_literal_action(decimal_integer_literal(), "909z", 3);
    assert_default_integer_literal_action(hexadecimal_integer_literal(), "0xf0fz", 5);
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
    let options_builder = |o: IntegerLiteralBodyOptions<MockAccumulator, MockAccumulator>| {
      o.separator_with(|s| s.acc_to_vec()).value_to_string()
    };
    let options = IntegerLiteralBodyOptions::default()
      .separator_with(|s| s.acc_to_vec())
      .value_to_string();

    assert_integer_literal_action(
      binary_integer_literal_with(&options_builder),
      "0b1_01z",
      "101",
    );
    assert_integer_literal_action(
      binary_integer_literal_with_options(options.clone()),
      "0b1_01z",
      "101",
    );
    assert_integer_literal_action(
      octal_integer_literal_with(&options_builder),
      "0o7_07z",
      "707",
    );
    assert_integer_literal_action(
      octal_integer_literal_with_options(options.clone()),
      "0o7_07z",
      "707",
    );
    assert_integer_literal_action(
      decimal_integer_literal_with(&options_builder),
      "9_0909z",
      "90909",
    );
    assert_integer_literal_action(
      decimal_integer_literal_with_options(options.clone()),
      "9_0909z",
      "90909",
    );
    assert_integer_literal_action(
      hexadecimal_integer_literal_with(&options_builder),
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
      let mut a = simple_with_data(|input| {
        let prefix = "0B";
        let suffix = "n"; // just like the big int literal in js/ts

        // check prefix
        if input.rest().starts_with(prefix) {
          // eat body
          let (digested, data) = binary_integer_literal_body(&input.rest()[prefix.len()..]);
          // check suffix
          if input.rest()[prefix.len() + digested..].starts_with(suffix) {
            Some((prefix.len() + digested + suffix.len(), data))
          } else {
            None
          }
        } else {
          None
        }
      });
      a.head_matcher = Some(HeadMatcher::OneOf(HashSet::from(['0'])));
      a
    };
    assert_default_integer_literal_action(action_factory(), "0B101n", 6);
    // missing suffix
    assert_reject(action_factory(), "0B101");
    // wrong prefix
    assert_reject(action_factory(), "0b101n");
  }
}
