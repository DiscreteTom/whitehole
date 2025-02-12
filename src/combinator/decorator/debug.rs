use super::create_value_decorator;
use crate::{
  action::{Action, Input, Output},
  combinator::Combinator,
  digest::Digest,
};
use std::{
  fmt::Debug,
  sync::atomic::{AtomicUsize, Ordering},
};

// TODO: don't make this generic
create_value_decorator!(Log, "See [`Combinator::log`].");

/// The indentation used in [`Combinator::log`].
pub static mut LOG_INDENTATION: &str = "| ";
static INDENT_LEVEL: AtomicUsize = AtomicUsize::new(0);
fn indentation() -> String {
  unsafe { LOG_INDENTATION }.repeat(INDENT_LEVEL.load(Ordering::Relaxed))
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
    &rest.chars().take(MAX_LEN).collect::<String>(),
    rest.chars().count() > MAX_LEN,
  )
}

macro_rules! impl_log {
  ($self:ident, $input:ident, $rest_formatter:ident) => {{
    let rest = $input.instant().rest();
    println!(
      "{}({}) input: {}",
      &indentation(),
      $self.inner,
      $rest_formatter(rest)
    );
    INDENT_LEVEL.fetch_add(1, Ordering::Relaxed);
    let output = $self.action.exec($input);
    INDENT_LEVEL.fetch_sub(1, Ordering::Relaxed);
    println!(
      "{}({}) output: {:?}",
      &indentation(),
      $self.inner,
      output
        .as_ref()
        .map(|o| { unsafe { rest.span_unchecked(o.digested) } }),
    );
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
  /// # fn t(combinator: Combinator<impl Action<str, MyState>>) {
  /// combinator.log("name")
  /// # ;}
  /// ```
  #[inline]
  pub fn log(self, name: &str) -> Combinator<Log<T, &str>> {
    Combinator::new(Log::new(self.action, name))
  }
}

// TODO: tests
