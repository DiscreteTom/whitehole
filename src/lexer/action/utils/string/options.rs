use super::{PartialStringBody, PartialStringBodyValue, StringBodyMatcher, StringBodyMatcherInput};

pub struct StringBodyOptions<Value = (), CustomError = (), ValueAcc = ()> {
  pub matchers: Vec<StringBodyMatcher<Value, CustomError>>,
  pub acc: ValueAcc,
  // TODO pub error_acc: ErrAcc,
}

impl<Value, CustomError, ValueAcc: Default> Default
  for StringBodyOptions<Value, CustomError, ValueAcc>
{
  fn default() -> Self {
    Self {
      matchers: Vec::new(),
      acc: ValueAcc::default(),
    }
  }
}

impl StringBodyOptions<(), (), ()> {
  // TODO: comments
  pub fn new() -> Self {
    Self::default()
  }
}

impl StringBodyOptions<String, (), ()> {
  // TODO: comments
  pub fn with_value() -> Self {
    Self::default()
  }
}

impl<CustomError> StringBodyOptions<(), CustomError, ()> {
  // TODO: comments
  pub fn with_error() -> Self {
    Self::default()
  }
}

impl<Value: PartialStringBodyValue, CustomError, ValueAcc>
  StringBodyOptions<Value, CustomError, ValueAcc>
{
  fn append_body_matcher(
    mut self,
    close: bool,
    matcher: impl Fn(&StringBodyMatcherInput) -> usize + 'static,
  ) -> Self {
    self
      .matchers
      .push(Box::new(move |input| match matcher(input) {
        0 => None,
        digested => Some(PartialStringBody {
          digested,
          value: if close {
            // the close quote doesn't count as the body
            Value::from_str("")
          } else {
            Value::from_str(&input.rest[..digested])
          },
          close,
          error: None,
        }),
      }));
    self
  }

  /// Append a string body matcher with a custom matcher function.
  /// The function should return how many bytes have been digested,
  /// or return `0` to indicate that the matcher does not match.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{StringBodyOptions};
  /// # let options =
  /// // accept all the rest of the input text as the string body
  /// StringBodyOptions::new().body(|input| input.rest.len());
  /// ```
  pub fn body(self, matcher: impl Fn(&StringBodyMatcherInput) -> usize + 'static) -> Self {
    self.append_body_matcher(false, matcher)
  }

  /// Append a string body matcher that consumes characters while the matcher function returns `true`.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{StringBodyOptions};
  /// # let options =
  /// // accept all the alphabetic characters as the string body
  /// StringBodyOptions::new().chars(|c| c.is_alphabetic());
  /// ```
  pub fn chars(self, matcher: impl Fn(char) -> bool + 'static) -> Self {
    self.body(move |input| {
      input
        .rest
        .chars()
        .take_while(|c| matcher(*c))
        .map(|c| c.len_utf8())
        .sum()
    })
  }

  /// Append a string body matcher with a custom matcher function.
  /// The matched part will be treated as the close quote of the string.
  /// The function should return how many bytes have been digested,
  /// or return `0` to indicate that the matcher does not match.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{StringBodyOptions};
  /// # let options =
  /// // accept "${" as the close quote of the string (interpolation string)
  /// StringBodyOptions::new()
  ///   .close_match(|input| if input.rest.starts_with("${") { 2 } else { 0 });
  /// ```
  pub fn close_match(self, matcher: impl Fn(&StringBodyMatcherInput) -> usize + 'static) -> Self {
    self.append_body_matcher(true, matcher)
  }

  /// Append a string body matcher that
  /// check if a character is the close quote of the string.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{StringBodyOptions};
  /// # let options =
  /// // accept `"` or `'` as the close quote of the string
  /// StringBodyOptions::new().close_if(|c| c == '"' || c == '\'');
  /// ```
  pub fn close_if(self, matcher: impl Fn(char) -> bool + 'static) -> Self {
    self.close_match(move |input| {
      if matcher(input.next) {
        input.next.len_utf8()
      } else {
        0
      }
    })
  }

  /// Append a string body matcher that
  /// check if a character is the close quote of the string.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{StringBodyOptions};
  /// # let options =
  /// // accept `"` as the close quote of the string
  /// StringBodyOptions::new().close('"');
  /// ```
  pub fn close(self, quote: char) -> Self {
    self.close_if(move |c| c == quote)
  }

  // TODO: comments
  pub fn acc<NewAcc>(self, acc: NewAcc) -> StringBodyOptions<Value, CustomError, NewAcc> {
    StringBodyOptions {
      matchers: self.matchers,
      acc,
    }
  }

  // TODO: comments
  pub fn acc_to_string(self) -> StringBodyOptions<Value, CustomError, String> {
    self.acc(String::new())
  }

  // TODO: comments
  pub fn acc_to_vec(self) -> StringBodyOptions<Value, CustomError, Vec<Value>> {
    self.acc(Vec::new())
  }
}
