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

#[inline]
fn indentation() -> String {
  LOG_INDENTATION.get().repeat(INDENT_LEVEL.get())
}

/// A trait to format the undigested text.
/// # Safety
/// The implementor must ensure the return value is valid according to [`Digest::validate`].
pub unsafe trait FormatUndigested {
  /// Return [`None`] if the text doesn't need to be truncated.
  /// Otherwise, return the number of bytes after truncation.
  fn truncated_len(&self) -> Option<usize>;
}

unsafe impl FormatUndigested for [u8] {
  fn truncated_len(&self) -> Option<usize> {
    let max = LOG_UNDIGESTED_MAX_LEN.get();
    if self.len() <= max {
      None
    } else {
      Some(max)
    }
  }
}

unsafe impl FormatUndigested for str {
  fn truncated_len(&self) -> Option<usize> {
    let max = LOG_UNDIGESTED_MAX_LEN.get();
    let mut len = 0;
    let mut chars = self.chars();
    for _ in 0..max {
      if let Some(c) = chars.next() {
        len += c.len_utf8();
      } else {
        return None;
      }
    }
    if chars.next().is_some() {
      Some(len)
    } else {
      None
    }
  }
}

#[inline]
fn format_input<Text: FormatUndigested + Digest + Debug + ?Sized>(name: &str, rest: &Text) -> String
where
  RangeTo<usize>: SliceIndex<Text, Output = Text>,
{
  let truncated = if let Some(len) = rest.truncated_len() {
    format!("{:?} (truncated)", unsafe { rest.get_unchecked(..len) })
  } else {
    format!("{:?}", rest)
  };

  format!("{}({}) input: {}", &indentation(), name, truncated)
}

#[inline]
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
    println!("{}", &format_input(self.name, rest));
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
  /// Be careful in multi-threaded environments since this uses thread-local variables.
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

  #[test]
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
  fn check_format_input() {
    INDENT_LEVEL.set(0);
    assert_eq!(format_input("name", "123"), "(name) input: \"123\"");
  }

  #[test]
  fn check_format_input_indent() {
    LOG_INDENTATION.set("| ");
    INDENT_LEVEL.set(1);
    assert_eq!(format_input("name", "123"), "| (name) input: \"123\"");
    INDENT_LEVEL.set(2);
    assert_eq!(format_input("name", "123"), "| | (name) input: \"123\"");
    // custom indentation
    LOG_INDENTATION.set("  ");
    assert_eq!(format_input("name", "123"), "    (name) input: \"123\"");
  }

  #[test]
  fn check_format_input_truncated() {
    INDENT_LEVEL.set(0);
    assert_eq!(
      format_input("name", "1234567890".repeat(10).as_str()),
      format!("(name) input: {:?}", "1234567890".repeat(10))
    );
    assert_eq!(
      format_input("name", "1234567890".repeat(11).as_str()),
      format!("(name) input: {:?} (truncated)", "1234567890".repeat(10))
    );
  }

  #[test]
  fn check_format_input_bytes() {
    INDENT_LEVEL.set(0);
    assert_eq!(
      format_input("name", b"123" as &[u8]),
      // TODO: prettier format bytes
      "(name) input: [49, 50, 51]"
    );
  }

  #[test]
  fn check_format_input_indent_bytes() {
    LOG_INDENTATION.set("| ");
    INDENT_LEVEL.set(1);
    assert_eq!(
      format_input("name", b"123" as &[u8]),
      "| (name) input: [49, 50, 51]"
    );
    INDENT_LEVEL.set(2);
    assert_eq!(
      format_input("name", b"123" as &[u8]),
      "| | (name) input: [49, 50, 51]"
    );
    // custom indentation
    LOG_INDENTATION.set("  ");
    assert_eq!(
      format_input("name", b"123" as &[u8]),
      "    (name) input: [49, 50, 51]"
    );
  }

  #[test]
  fn check_format_input_truncated_bytes() {
    INDENT_LEVEL.set(0);
    assert_eq!(
      format_input("name", b"1234567890".repeat(10).as_slice()),
      format!("(name) input: {:?}", b"1234567890".repeat(10).as_slice())
    );
    assert_eq!(
      format_input("name", b"1234567890".repeat(11).as_slice()),
      format!(
        "(name) input: {:?} (truncated)",
        b"1234567890".repeat(10).as_slice()
      )
    );
  }

  #[test]
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

  fn _ensure_debug() {
    let _ = format!("{:?}", take(1).log("take"));
  }

  fn _ensure_clone_copy() {
    let c = take(1).log("take");
    let _c = c;
    let _ = c.clone();
  }
}
