use super::core::lex::expectation::Expectation;

pub struct LexerLexOptions<'input_text, 'expect, Kind> {
  pub input: Option<&'input_text str>,
  pub peek: bool,
  pub expectation: Expectation<'expect, Kind>,
}

impl<'input, 'expect, Kind> Default for LexerLexOptions<'input, 'expect, Kind> {
  fn default() -> Self {
    LexerLexOptions {
      input: None,
      peek: false,
      expectation: Expectation::default(),
    }
  }
}

impl<'input, 'expect, Kind> LexerLexOptions<'input, 'expect, Kind> {
  pub fn input(mut self, input: impl Into<&'input str>) -> Self {
    self.input = Some(input.into());
    self
  }

  pub fn peek(mut self, peek: impl Into<bool>) -> Self {
    self.peek = peek.into();
    self
  }

  /// ```
  /// // expect token's text content
  /// LexOptions::default().expect("abc");
  ///
  /// # #[derive(TokenKind)]
  /// # enum MyKind {
  /// #   UnitField,
  /// #   UnnamedField(i32),
  /// #   NamedField { _a: i32 },
  /// # }
  /// // expect token's kind
  /// LexOptions::default().expect(MyKind::UnitField);
  ///
  /// // expect both token's text and kind
  /// LexOptions::default().expect("abc").expect(MyKind::UnitField);
  /// ```
  pub fn expect(mut self, expectation: impl Into<Expectation<'expect, Kind>>) -> Self {
    let Expectation { text, kind } = expectation.into();
    if let Some(text) = text {
      self.expectation.text = Some(text);
    }
    if let Some(kind) = kind {
      self.expectation.kind = Some(kind);
    }
    self
  }
}
