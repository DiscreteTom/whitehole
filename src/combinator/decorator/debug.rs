use super::create_value_decorator;
use crate::{
  action::{Action, Input, Output},
  combinator::Combinator,
  digest::Digest,
};
use std::fmt::Debug;

// TODO: don't make this generic
create_value_decorator!(Log, "See [`Combinator::log`].");

/// The indentation used in [`Combinator::log`].
pub static mut LOG_INDENTATION: &str = "| ";
static mut INDENT_LEVEL: usize = 0;
#[allow(static_mut_refs)] // TODO: find a better way to do this
fn indentation() -> String {
  unsafe { LOG_INDENTATION.repeat(INDENT_LEVEL) }
}

const MAX_LEN: usize = 100;
fn format_truncated(truncated: impl Debug, oversize: bool) -> String {
  if oversize {
    format!("{:?} (truncated)", truncated)
  } else {
    format!("{:?}", truncated)
  }
}
fn input_rest_bytes(rest: &[u8]) -> String {
  format_truncated(&rest[..rest.len().min(MAX_LEN)], rest.len() > MAX_LEN)
}
fn input_rest_str(rest: &str) -> String {
  format_truncated(
    rest.chars().take(MAX_LEN).collect::<String>(),
    rest.chars().count() > MAX_LEN,
  )
}

fn format_input<TextRef>(
  name: &str,
  rest_formatter: fn(TextRef) -> String,
  rest: TextRef,
) -> String {
  format!(
    "{}({}) input: {}",
    &indentation(),
    name,
    rest_formatter(rest)
  )
}

fn format_output<Text: ?Sized + Digest + Debug, Value>(
  name: &str,
  rest: &Text,
  output: &Option<Output<Value>>,
) -> String {
  format!(
    "{}({}) output: {:?}",
    &indentation(),
    name,
    output
      .as_ref()
      .map(|o| { unsafe { rest.span_unchecked(o.digested) } }),
  )
}

macro_rules! impl_log {
  ($self:ident, $input:ident, $rest_formatter:ident) => {{
    let rest = $input.instant().rest();
    println!("{}", &format_input($self.inner, $rest_formatter, rest));
    unsafe { INDENT_LEVEL += 1 };
    let output = $self.action.exec($input);
    unsafe { INDENT_LEVEL -= 1 };
    println!("{}", &format_output($self.inner, rest, &output));
    output
  }};
}

unsafe impl<State, Heap, T: Action<str, State, Heap>> Action<str, State, Heap> for Log<T, &str> {
  type Value = T::Value;

  #[inline]
  fn exec(&self, input: Input<&str, &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    impl_log!(self, input, input_rest_str)
  }
}

unsafe impl<State, Heap, T: Action<[u8], State, Heap>> Action<[u8], State, Heap> for Log<T, &str> {
  type Value = T::Value;

  #[inline]
  fn exec(&self, input: Input<&[u8], &mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    impl_log!(self, input, input_rest_bytes)
  }
}

impl<T> Combinator<T> {
  /// Create a new combinator to print the input text
  /// and the digested text by the action.
  /// # Caveats
  /// This should NOT be used in multi-threaded environments
  /// because it uses a global variable to store the indentation level.
  /// # Examples
  /// ```
  /// # use whitehole::{action::Action, combinator::Combinator};
  /// # fn t(combinator: Combinator<impl Action>) {
  /// combinator.log("name")
  /// # ;}
  /// ```
  #[inline]
  pub fn log(self, name: &str) -> Combinator<Log<T, &str>> {
    Combinator::new(Log::new(self.action, name))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    combinator::{bytes, wrap},
    instant::Instant,
  };
  use serial_test::serial;

  #[test]
  #[serial]
  fn ensure_log_does_not_modify_output() {
    let c = wrap(|input| input.digest(1)).bind(2).log("name");
    let output = c
      .exec(Input::new(Instant::new("1"), &mut (), &mut ()))
      .unwrap();
    assert_eq!(output.digested, 1);
    assert_eq!(output.value, 2);
  }

  #[test]
  #[serial]
  fn ensure_log_can_be_used_with_bytes() {
    let c = bytes::wrap(|input| input.digest(1)).bind(2).log("name");
    let output = c
      .exec(Input::new(Instant::new(b"1"), &mut (), &mut ()))
      .unwrap();
    assert_eq!(output.digested, 1);
    assert_eq!(output.value, 2);
  }

  #[test]
  #[serial]
  fn check_format_input() {
    unsafe { INDENT_LEVEL = 0 };
    assert_eq!(
      format_input("name", input_rest_str, "123"),
      "(name) input: \"123\""
    );
  }

  #[test]
  #[serial]
  fn check_format_input_indent() {
    unsafe { LOG_INDENTATION = "| " };
    unsafe { INDENT_LEVEL = 1 };
    assert_eq!(
      format_input("name", input_rest_str, "123"),
      "| (name) input: \"123\""
    );
    unsafe { INDENT_LEVEL = 2 };
    assert_eq!(
      format_input("name", input_rest_str, "123"),
      "| | (name) input: \"123\""
    );
    // custom indentation
    unsafe { LOG_INDENTATION = "  " };
    assert_eq!(
      format_input("name", input_rest_str, "123"),
      "    (name) input: \"123\""
    );
  }

  #[test]
  #[serial]
  fn check_format_input_truncated() {
    unsafe { INDENT_LEVEL = 0 };
    assert_eq!(
      format_input("name", input_rest_str, &"1234567890".repeat(10)),
      "(name) input: \"1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890\""
    );
    assert_eq!(
      format_input("name", input_rest_str, &"1234567890".repeat(11)),
      "(name) input: \"1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890\" (truncated)"
    );
  }

  #[test]
  #[serial]
  fn check_format_input_bytes() {
    unsafe { INDENT_LEVEL = 0 };
    assert_eq!(
      format_input("name", input_rest_bytes, b"123"),
      "(name) input: [49, 50, 51]"
    );
  }

  #[test]
  #[serial]
  fn check_format_input_indent_bytes() {
    unsafe { LOG_INDENTATION = "| " };
    unsafe { INDENT_LEVEL = 1 };
    assert_eq!(
      format_input("name", input_rest_bytes, b"123"),
      "| (name) input: [49, 50, 51]"
    );
    unsafe { INDENT_LEVEL = 2 };
    assert_eq!(
      format_input("name", input_rest_bytes, b"123"),
      "| | (name) input: [49, 50, 51]"
    );
    // custom indentation
    unsafe { LOG_INDENTATION = "  " };
    assert_eq!(
      format_input("name", input_rest_bytes, b"123"),
      "    (name) input: [49, 50, 51]"
    );
  }

  #[test]
  #[serial]
  fn check_format_input_truncated_bytes() {
    unsafe { INDENT_LEVEL = 0 };
    assert_eq!(
      format_input("name", input_rest_bytes, &b"1234567890".repeat(10)),
      "(name) input: [49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48]"
    );
    assert_eq!(
      format_input("name", input_rest_bytes, &b"1234567890".repeat(11)),
      "(name) input: [49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48] (truncated)"
    );
  }

  #[test]
  #[serial]
  fn check_format_output() {
    unsafe { INDENT_LEVEL = 0 };
    assert_eq!(
      format_output(
        "name",
        "123",
        &Some(Output {
          value: (),
          digested: 1
        })
      ),
      "(name) output: Some(\"1\")"
    );
    assert_eq!(
      format_output::<_, ()>("name", "123", &None),
      "(name) output: None"
    );
  }

  #[test]
  #[serial]
  fn check_format_output_bytes() {
    unsafe { INDENT_LEVEL = 0 };
    assert_eq!(
      format_output(
        "name",
        b"123" as &[u8],
        &Some(Output {
          value: (),
          digested: 1
        })
      ),
      "(name) output: Some([49])"
    );
    assert_eq!(
      format_output::<_, ()>("name", b"123" as &[u8], &None),
      "(name) output: None"
    );
  }
}
