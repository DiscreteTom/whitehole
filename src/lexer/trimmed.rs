use super::{
  expectation::Expectation,
  output::{LexOutput, PeekOutput},
  state::LexerState,
  stateless::StatelessLexer,
  token::{Token, TokenKind},
  Lexer,
};

pub struct TrimmedLexerLexOutput<TokenType, Lexer> {
  pub token: Option<TokenType>,
  pub digested: usize,
  pub errors: Vec<TokenType>,
  pub lexer: Lexer,
}

pub struct TrimmedLexerLexAllOutput<TokenType, Lexer> {
  pub tokens: Vec<TokenType>,
  pub digested: usize,
  pub errors: Vec<TokenType>,
  pub lexer: Lexer,
}

pub struct TrimmedLexer<'buffer, Kind: 'static, ActionState: 'static, ErrorType: 'static>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  /// The lexer should always be trimmed.
  lexer: Lexer<'buffer, Kind, ActionState, ErrorType>,
}

impl<'buffer, Kind: 'static, ActionState: 'static, ErrorType: 'static>
  Into<Lexer<'buffer, Kind, ActionState, ErrorType>>
  for TrimmedLexer<'buffer, Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  fn into(self) -> Lexer<'buffer, Kind, ActionState, ErrorType> {
    self.lexer
  }
}

impl<'buffer, Kind: 'static, ActionState: 'static, ErrorType: 'static>
  From<Lexer<'buffer, Kind, ActionState, ErrorType>>
  for TrimmedLexer<'buffer, Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  fn from(mut lexer: Lexer<'buffer, Kind, ActionState, ErrorType>) -> Self {
    lexer.trim();
    TrimmedLexer { lexer }
  }
}

impl<'buffer, Kind: 'static, ActionState: 'static, ErrorType: 'static> Clone
  for TrimmedLexer<'buffer, Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  fn clone(&self) -> Self {
    TrimmedLexer {
      lexer: self.lexer.clone(),
    }
  }
}

impl<'buffer, Kind: 'static, ActionState: 'static, ErrorType: 'static>
  TrimmedLexer<'buffer, Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  pub fn new(lexer: Lexer<'buffer, Kind, ActionState, ErrorType>) -> Self {
    TrimmedLexer { lexer }
  }

  pub fn stateless(&self) -> &StatelessLexer<Kind, ActionState, ErrorType> {
    self.lexer.stateless()
  }
  pub fn state(&self) -> &LexerState<'buffer> {
    self.lexer.state()
  }
  pub fn action_state(&self) -> &ActionState {
    self.lexer.action_state()
  }
  pub fn action_state_mut(&mut self) -> &mut ActionState {
    self.lexer.action_state_mut()
  }

  pub fn reload<'new_buffer>(
    self,
    buffer: &'new_buffer str,
  ) -> Lexer<'new_buffer, Kind, ActionState, ErrorType> {
    // load a new buffer, the result is not a trimmed lexer
    self.lexer.reload(buffer)
  }

  pub fn clone_with<'new_buffer>(
    &self,
    buffer: &'new_buffer str,
  ) -> Lexer<'new_buffer, Kind, ActionState, ErrorType> {
    // load a new buffer, the result is not a trimmed lexer
    self.lexer.clone_with(buffer)
  }

  pub fn peek(&self) -> PeekOutput<Token<'buffer, Kind, ErrorType>, ActionState> {
    self.lexer.peek()
  }

  pub fn peek_expect<'expect_text>(
    &self,
    expectation: impl Into<Expectation<'expect_text, Kind>>,
  ) -> PeekOutput<Token<'buffer, Kind, ErrorType>, ActionState> {
    self.lexer.peek_expect(expectation)
  }

  // this will mutate the lexer's state
  // so consume the trimmed lexer, yield a lexer
  // TODO: rename to `into_lexed`
  pub fn lex(
    self,
  ) -> TrimmedLexerLexOutput<
    Token<'buffer, Kind, ErrorType>,
    Lexer<'buffer, Kind, ActionState, ErrorType>,
  > {
    self.lex_expect(Expectation::default())
  }

  // this will mutate the lexer's state
  // so consume the trimmed lexer, yield a lexer
  pub fn lex_expect<'expect_text>(
    mut self,
    expectation: impl Into<Expectation<'expect_text, Kind>>,
  ) -> TrimmedLexerLexOutput<
    Token<'buffer, Kind, ErrorType>,
    Lexer<'buffer, Kind, ActionState, ErrorType>,
  > {
    let output = self.lexer.lex_expect(expectation);
    TrimmedLexerLexOutput {
      token: output.token,
      digested: output.digested,
      errors: output.errors,
      lexer: self.lexer,
    }
  }

  // this will mutate the lexer's state
  // so consume the trimmed lexer, yield a lexer
  pub fn lex_all(
    mut self,
  ) -> TrimmedLexerLexAllOutput<
    Token<'buffer, Kind, ErrorType>,
    Lexer<'buffer, Kind, ActionState, ErrorType>,
  > {
    let output = self.lexer.lex_all();
    TrimmedLexerLexAllOutput {
      tokens: output.tokens,
      digested: output.digested,
      errors: output.errors,
      lexer: self.lexer,
    }
  }

  // this will mutate the lexer's state
  // so consume the trimmed lexer, yield a lexer
  // TODO: rename to `into_taken`
  pub fn take(
    mut self,
    n: usize,
    state: Option<ActionState>,
  ) -> Lexer<'buffer, Kind, ActionState, ErrorType> {
    self.lexer.take(n, state);
    self.lexer
  }

  /// Apply a function to the inner lexer.
  /// After that the inner lexer will be trimmed.
  pub fn apply<F, R>(&mut self, f: F) -> R
  where
    F: FnOnce(&mut Lexer<'buffer, Kind, ActionState, ErrorType>) -> R,
  {
    let res = f(&mut self.lexer);
    self.lexer.trim();
    res
  }

  // TODO: rename to `take`
  pub fn take_and_trim(&mut self, n: usize, state: Option<ActionState>) -> &mut Self {
    self.apply(|lexer| {
      lexer.take(n, state);
    });
    self
  }

  // TODO: rename to `lex`
  pub fn lex_and_trim(&mut self) -> LexOutput<Token<'buffer, Kind, ErrorType>> {
    self.apply(|lexer| lexer.lex())
  }

  // there is no `trim` or `into_trimmed` for TrimmedLexer
}
