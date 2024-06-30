use super::Error;

pub struct StringBodyMatcherInput<'text> {
  /// The whole input text.
  pub text: &'text str,
  /// The start index in bytes.
  /// This is guaranteed to be smaller than the length of [`Self::text`].
  pub start: usize,
  /// The rest of the input text.
  /// This is guaranteed to be non-empty.
  // this is precalculated and cached because this might be used for at least once
  // when traversing string body matchers
  pub rest: &'text str,

  // private field to prevent users from constructing this struct
  __: (),
}

impl<'text> StringBodyMatcherInput<'text> {
  /// Return [`None`] if the [`start`](Self::start) position is out of the input
  /// [`text`](Self::text) or there is no [`rest`](Self::rest).
  pub fn new(text: &'text str, start: usize) -> Option<Self> {
    if start < text.len() {
      Some(Self {
        text,
        start,
        rest: &text[start..],
        __: (),
      })
    } else {
      None
    }
  }
}

pub struct PartialStringBody<Value, CustomError> {
  /// The number of bytes that have been digested as the partial string body.
  pub digested: usize,
  /// The value of the partial string body.
  pub value: Value,
  /// Whether the partial string body contains a close quote.
  /// If `true`, the lexer will stop lexing the string.
  pub close: bool,
  /// The error that occurred during lexing the partial string body.
  pub error: Option<Error<CustomError>>,
}

pub type StringBodyMatcher<Value, CustomError> =
  Box<dyn Fn(&StringBodyMatcherInput) -> Option<PartialStringBody<Value, CustomError>>>;
