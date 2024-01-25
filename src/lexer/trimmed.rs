use super::{
  expectation::Expectation,
  token::{Token, TokenKind},
  Lexer,
};
use std::rc::Rc;

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

#[derive(Clone)]
pub struct TrimmedLexer<'buffer, Kind: 'static, ActionState: 'static, ErrorType: 'static>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  lexer: Lexer<'buffer, Kind, ActionState, ErrorType>,
}

impl<'buffer, Kind: 'static, ActionState: 'static, ErrorType: 'static>
  TrimmedLexer<'buffer, Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  pub fn new(lexer: Lexer<'buffer, Kind, ActionState, ErrorType>) -> Self {
    TrimmedLexer { lexer }
  }

  pub fn dry_clone<'new_buffer>(
    &self,
    buffer: &'new_buffer str,
  ) -> Lexer<'new_buffer, Kind, ActionState, ErrorType> {
    self.lexer.dry_clone(buffer)
  }

  pub fn rest(&self) -> &'buffer str {
    self.lexer.rest()
  }

  pub fn lex(
    self,
  ) -> TrimmedLexerLexOutput<
    Rc<Token<'buffer, Kind, ErrorType>>,
    Lexer<'buffer, Kind, ActionState, ErrorType>,
  > {
    self.lex_expect(Expectation::default())
  }

  pub fn lex_expect<'expect_text>(
    mut self,
    expectation: impl Into<Expectation<'expect_text, Kind>>,
  ) -> TrimmedLexerLexOutput<
    Rc<Token<'buffer, Kind, ErrorType>>,
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
    Rc<Token<'buffer, Kind, ErrorType>>,
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
}
