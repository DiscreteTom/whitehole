use crate::{
  action::{Action, Input, Output},
  combinator::Combinator,
  digest::Digest,
  instant::Instant,
};
use std::{cell::Cell, fmt::Debug, ops::RangeTo, slice::SliceIndex};

/// See [`Combinator::log`].
#[derive(Copy, Clone, Debug)]
pub struct Log<'a, T> {
  action: T,
  name: &'a str,
}

impl<'a, T> Log<'a, T> {
  #[inline]
  const fn new(action: T, name: &'a str) -> Self {
    Self { action, name }
  }
}

thread_local! {
  /// The indentation used in [`Combinator::log`].
  pub static LOG_INDENTATION: Cell<&str> = const { Cell::new("| ") };

  /// The max length of the undigested text.
  /// If the actual length is greater than this, it will be truncated.
  pub static LOG_UNDIGESTED_MAX_LEN: Cell<usize> = const { Cell::new(100) };

  static INDENT_LEVEL: Cell<usize> = const { Cell::new(0) };
}

fn indentation() -> String {
  LOG_INDENTATION.get().repeat(INDENT_LEVEL.get())
}

fn format_truncated(truncated: impl Debug, oversize: bool) -> String {
  if oversize {
    format!("{:?} (truncated)", truncated)
  } else {
    format!("{:?}", truncated)
  }
}

/// A trait to format the rest input text.
pub trait FormatUndigested {
  // TODO: redesign API

  /// Format the rest input text to a string.
  fn format_rest(&self) -> String;
}

impl FormatUndigested for [u8] {
  fn format_rest(&self) -> String {
    format_truncated(
      &self[..self.len().min(LOG_UNDIGESTED_MAX_LEN.get())],
      self.len() > LOG_UNDIGESTED_MAX_LEN.get(),
    )
  }
}
impl FormatUndigested for str {
  fn format_rest(&self) -> String {
    format_truncated(
      self
        .chars()
        .take(LOG_UNDIGESTED_MAX_LEN.get())
        .collect::<String>(),
      self.chars().count() > LOG_UNDIGESTED_MAX_LEN.get(),
    )
  }
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
) -> String
where
  RangeTo<usize>: SliceIndex<Text, Output = Text>,
{
  format!(
    "{}({}) output: {:?}",
    &indentation(),
    name,
    output.as_ref().and_then(|o| { rest.get(..o.digested) }),
  )
}

unsafe impl<T: Action<Text: FormatUndigested + Digest + Debug>> Action for Log<'_, T>
where
  RangeTo<usize>: SliceIndex<T::Text, Output = T::Text>,
{
  type Text = T::Text;
  type State = T::State;
  type Heap = T::Heap;
  type Value = T::Value;

  #[inline]
  fn exec(
    &self,
    input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    let rest = input.instant.rest();
    println!(
      "{}",
      &format_input(self.name, <T::Text as FormatUndigested>::format_rest, rest)
    );
    INDENT_LEVEL.set(INDENT_LEVEL.get() + 1);
    let output = self.action.exec(input);
    INDENT_LEVEL.set(INDENT_LEVEL.get() - 1);
    println!("{}", &format_output(self.name, rest, &output));
    output
  }
}

impl<T> Combinator<T> {
  /// Create a new combinator to print the input text
  /// and the digested text by the action.
  ///
  /// For customization, see [`LOG_INDENTATION`] and [`LOG_UNDIGESTED_MAX_LEN`].
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
  pub fn log(self, name: &str) -> Combinator<Log<T>> {
    Combinator::new(Log::new(self.action, name))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    combinator::{bytes, take},
    instant::Instant,
  };
  use serial_test::serial;

  #[test]
  #[serial]
  fn ensure_log_does_not_modify_output() {
    let c = take(1).bind(2).log("name");
    let output = c
      .exec(Input {
        instant: &Instant::new("1"),
        state: &mut (),
        heap: &mut (),
      })
      .unwrap();
    assert_eq!(output.digested, 1);
    assert_eq!(output.value, 2);
  }

  #[test]
  #[serial]
  fn ensure_log_can_be_used_with_bytes() {
    let c = bytes::take(1).bind(2).log("name");
    let output = c
      .exec(Input {
        instant: &Instant::new(b"1" as &[u8]),
        state: &mut (),
        heap: &mut (),
      })
      .unwrap();
    assert_eq!(output.digested, 1);
    assert_eq!(output.value, 2);
  }

  #[test]
  #[serial]
  fn check_format_input() {
    INDENT_LEVEL.set(0);
    assert_eq!(
      format_input("name", <str as FormatUndigested>::format_rest, "123"),
      "(name) input: \"123\""
    );
  }

  #[test]
  #[serial]
  fn check_format_input_indent() {
    LOG_INDENTATION.set("| ");
    INDENT_LEVEL.set(1);
    assert_eq!(
      format_input("name", <str as FormatUndigested>::format_rest, "123"),
      "| (name) input: \"123\""
    );
    INDENT_LEVEL.set(2);
    assert_eq!(
      format_input("name", <str as FormatUndigested>::format_rest, "123"),
      "| | (name) input: \"123\""
    );
    // custom indentation
    LOG_INDENTATION.set("  ");
    assert_eq!(
      format_input("name", <str as FormatUndigested>::format_rest, "123"),
      "    (name) input: \"123\""
    );
  }

  #[test]
  #[serial]
  fn check_format_input_truncated() {
    INDENT_LEVEL.set(0);
    assert_eq!(
      format_input("name", <str as FormatUndigested>::format_rest, &"1234567890".repeat(10)),
      "(name) input: \"1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890\""
    );
    assert_eq!(
      format_input("name", <str as FormatUndigested>::format_rest, &"1234567890".repeat(11)),
      "(name) input: \"1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890\" (truncated)"
    );
  }

  #[test]
  #[serial]
  fn check_format_input_bytes() {
    INDENT_LEVEL.set(0);
    assert_eq!(
      format_input("name", <[u8] as FormatUndigested>::format_rest, b"123"),
      "(name) input: [49, 50, 51]"
    );
  }

  #[test]
  #[serial]
  fn check_format_input_indent_bytes() {
    LOG_INDENTATION.set("| ");
    INDENT_LEVEL.set(1);
    assert_eq!(
      format_input("name", <[u8] as FormatUndigested>::format_rest, b"123"),
      "| (name) input: [49, 50, 51]"
    );
    INDENT_LEVEL.set(2);
    assert_eq!(
      format_input("name", <[u8] as FormatUndigested>::format_rest, b"123"),
      "| | (name) input: [49, 50, 51]"
    );
    // custom indentation
    LOG_INDENTATION.set("  ");
    assert_eq!(
      format_input("name", <[u8] as FormatUndigested>::format_rest, b"123"),
      "    (name) input: [49, 50, 51]"
    );
  }

  #[test]
  #[serial]
  fn check_format_input_truncated_bytes() {
    INDENT_LEVEL.set(0);
    assert_eq!(
      format_input("name", <[u8] as FormatUndigested>::format_rest, &b"1234567890".repeat(10)),
      "(name) input: [49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48]"
    );
    assert_eq!(
      format_input("name", <[u8] as FormatUndigested>::format_rest, &b"1234567890".repeat(11)),
      "(name) input: [49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48] (truncated)"
    );
  }

  #[test]
  #[serial]
  fn check_format_output() {
    INDENT_LEVEL.set(0);
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
    INDENT_LEVEL.set(0);
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
