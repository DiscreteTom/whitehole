use super::{
  PartialStringBody, PartialStringBodyValue, StringBodyMatcher, StringBodyMatcherInput,
  StringLiteralError,
};

pub struct StringBodyOptions<Value = (), CustomError = (), BodyAcc = ()> {
  pub matchers: Vec<StringBodyMatcher<Value, CustomError>>,
  pub acc: BodyAcc,
}

impl<Value, CustomError> Default for StringBodyOptions<Value, CustomError, ()> {
  fn default() -> Self {
    Self {
      matchers: Vec::new(),
      acc: (),
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

impl<Value: PartialStringBodyValue, CustomError, BodyAcc>
  StringBodyOptions<Value, CustomError, BodyAcc>
{
  fn append_body_matcher(
    mut self,
    matcher: impl Fn(&StringBodyMatcherInput) -> usize + 'static,
    handler: impl Fn(usize, &StringBodyMatcherInput) -> PartialStringBody<Value, CustomError> + 'static,
  ) -> Self {
    self
      .matchers
      .push(Box::new(move |input| match matcher(input) {
        0 => None,
        digested => Some(handler(digested, input)),
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
    self.append_body_matcher(matcher, |digested, input| PartialStringBody {
      digested,
      value: Value::from_str(&input.rest[..digested]),
      close: false,
      error: None,
    })
  }

  /// Append a string body matcher that consumes characters while the matcher function returns `true`.
  /// # Caveats
  /// This matcher is greedy, it will try to match as many characters as possible as one partial string body
  /// to optimize the performance. If you use `chars(|_| true)`, it will consume the whole input text.
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
    self.append_body_matcher(matcher, |digested, _input| PartialStringBody {
      digested,
      value: Value::default(),
      close: true,
      error: None,
    })
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
    self.close_match(char_matcher_to_body_matcher(matcher))
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
  pub fn unterminated_match(
    self,
    matcher: impl Fn(&StringBodyMatcherInput) -> usize + 'static,
    inclusive: bool,
  ) -> Self {
    self.append_body_matcher(matcher, move |digested, input| PartialStringBody {
      digested: if inclusive { digested } else { 0 },
      value: if inclusive {
        Value::from_str(&input.rest[..digested])
      } else {
        Value::default()
      },
      close: true,
      error: Some(StringLiteralError::Unterminated),
    })
  }

  // TODO: comments
  pub fn unterminated_if(self, matcher: impl Fn(char) -> bool + 'static, inclusive: bool) -> Self {
    self.unterminated_match(char_matcher_to_body_matcher(matcher), inclusive)
  }

  // TODO: comments
  pub fn unterminated(self, boundary: char, inclusive: bool) -> Self {
    self.unterminated_if(move |c| c == boundary, inclusive)
  }

  // TODO: comments
  pub fn singleline(self) -> Self {
    self.unterminated('\n', false).unterminated('\r', false)
  }

  // TODO: comments
  pub fn acc<NewAcc>(self, acc: NewAcc) -> StringBodyOptions<Value, CustomError, NewAcc> {
    StringBodyOptions {
      matchers: self.matchers,
      acc,
    }
  }

  // TODO: comments
  pub fn acc_to_vec(
    self,
  ) -> StringBodyOptions<Value, CustomError, Vec<PartialStringBody<Value, CustomError>>> {
    self.acc(Vec::new())
  }
}

impl<CustomError, BodyAcc> StringBodyOptions<String, CustomError, BodyAcc> {
  // TODO: comments
  pub fn acc_to_string(self) -> StringBodyOptions<String, CustomError, String> {
    self.acc(String::new())
  }
}

fn char_matcher_to_body_matcher(
  matcher: impl Fn(char) -> bool + 'static,
) -> impl Fn(&StringBodyMatcherInput) -> usize + 'static {
  move |input| {
    if matcher(input.next) {
      input.next.len_utf8()
    } else {
      0
    }
  }
}
