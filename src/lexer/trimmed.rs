use super::{
  expectation::Expectation,
  output::PeekOutput,
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
  lexer: Lexer<'buffer, Kind, ActionState, ErrorType>,
}

impl<'buffer, Kind: 'static, ActionState: 'static, ErrorType: 'static>
  From<Lexer<'buffer, Kind, ActionState, ErrorType>>
  for TrimmedLexer<'buffer, Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  fn from(lexer: Lexer<'buffer, Kind, ActionState, ErrorType>) -> Self {
    lexer.into_trimmed().trimmed_lexer
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
    &self.lexer.stateless
  }
  pub fn state(&self) -> &LexerState<'buffer> {
    &self.lexer.state
  }
  pub fn action_state(&self) -> &ActionState {
    &self.lexer.action_state
  }
  // user can mutate the action state
  pub fn action_state_mut(&mut self) -> &mut ActionState {
    &mut self.lexer.action_state
  }

  pub fn reload<'new_buffer>(
    self,
    buffer: &'new_buffer str,
  ) -> Lexer<'new_buffer, Kind, ActionState, ErrorType> {
    // load a new buffer, so the result is not a trimmed lexer
    self.lexer.reload(buffer)
  }

  pub fn clone_with<'new_buffer>(
    &self,
    buffer: &'new_buffer str,
  ) -> Lexer<'new_buffer, Kind, ActionState, ErrorType> {
    // load a new buffer, so the result is not a trimmed lexer
    self.lexer.clone_with(buffer)
  }

  pub fn rest(&self) -> &'buffer str {
    self.lexer.rest()
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

  pub fn lex(
    self,
  ) -> TrimmedLexerLexOutput<
    Token<'buffer, Kind, ErrorType>,
    Lexer<'buffer, Kind, ActionState, ErrorType>,
  > {
    self.lex_expect(Expectation::default())
  }

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

  pub fn take(
    mut self,
    n: usize,
    state: Option<ActionState>,
  ) -> Lexer<'buffer, Kind, ActionState, ErrorType> {
    self.lexer.take(n, state);
    self.lexer
  }
}
