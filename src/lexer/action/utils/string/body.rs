use super::{PartialStringBodyValue, StringBodyOptions, StringLiteralError};
use crate::lexer::action::Accumulator;

pub struct StringBodyMatcherInput<'text> {
  /// The rest of the input text.
  /// This is guaranteed to be non-empty.
  // this is precalculated and cached because this might be used for at least once
  // when traversing string body matchers
  pub rest: &'text str,
  /// The next char in the rest of the input text.
  // this is precalculated and cached because this might be used for at least once
  // when traversing string body matchers
  pub next: char,

  // private field to prevent users from constructing this struct
  __: (),
}

impl<'text> StringBodyMatcherInput<'text> {
  /// Return [`None`] if [`rest`](Self::rest) is empty.
  pub fn new(rest: &'text str) -> Option<Self> {
    rest.chars().next().map(|next| Self { rest, next, __: () })
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
  pub error: Option<StringLiteralError<CustomError>>,
}

impl<CustomError> Accumulator<PartialStringBody<String, CustomError>> for String {
  fn update(&mut self, t: PartialStringBody<String, CustomError>) {
    self.push_str(&t.value);
  }
}

pub type StringBodyMatcher<Value, CustomError> =
  Box<dyn Fn(&StringBodyMatcherInput) -> Option<PartialStringBody<Value, CustomError>>>;

// TODO: comments
pub fn string_body<
  Value: PartialStringBodyValue,
  CustomError,
  BodyAcc: Accumulator<PartialStringBody<Value, CustomError>> + Clone,
>(
  rest: &str,
  options: &StringBodyOptions<Value, CustomError, BodyAcc>,
) -> (usize, BodyAcc) {
  let mut digested = 0;
  let mut acc = options.acc.clone();

  'outer: while let Some(input) = StringBodyMatcherInput::new(&rest[digested..]) {
    for m in &options.matchers {
      if let Some(partial) = m(&input) {
        digested += partial.digested;
        let close = partial.close;

        acc.update(partial);

        if close {
          break 'outer;
        }

        // break the for-loop, construct new input and continue
        break;
      };
      // else, continue to the next matcher
    }

    // no matcher matches, mark as unterminated and stop lexing
    acc.update(PartialStringBody {
      digested: 0,
      value: Value::from_str(""),
      close: false,
      error: Some(StringLiteralError::Unterminated),
    });
    break;
  }

  (digested, acc)
}
