pub mod action;
pub mod builder;
pub mod core;
pub mod options;
pub mod state;
pub mod token;

use self::{
  action::Action,
  core::{
    lex::{options::LexerCoreLexOptions, LexAllOutput, LexOutput},
    trim::TrimOutput,
    LexerCore,
  },
  options::LexerLexOptions,
  state::LexerState,
  token::{Token, TokenKind},
};
use std::rc::Rc;

#[derive(Clone)]
pub struct Lexer<'buffer, Kind: 'static, ActionState: 'static, ErrorType: 'static>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  core: LexerCore<Kind, ActionState, ErrorType>,
  state: LexerState<'buffer>,
}

impl<'buffer, Kind: 'static, ActionState: 'static, ErrorType: 'static>
  Lexer<'buffer, Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  pub fn new(actions: Vec<Action<Kind, ActionState, ErrorType>>, buffer: &'buffer str) -> Self {
    Lexer {
      core: LexerCore::new(actions),
      state: LexerState::new(buffer),
    }
  }

  pub fn rest(&self) -> &str {
    &self.state.buffer()[self.state.digested()..]
  }

  pub fn lex(&mut self) -> LexOutput<Rc<Token<'buffer, Kind, ErrorType>>> {
    self.lex_with(LexerLexOptions::default())
  }

  pub fn lex_with<'expect>(
    &mut self,
    options: impl Into<LexerLexOptions<'expect, Kind>>,
  ) -> LexOutput<Rc<Token<'buffer, Kind, ErrorType>>> {
    let options: LexerLexOptions<Kind> = options.into();

    let res = self.core.lex(
      self.state.buffer(),
      LexerCoreLexOptions {
        start: self.state.digested(),
        peek: options.peek,
        expectation: options.expectation,
      },
    );

    // update state if not peek
    if !options.peek {
      self.state.digest(res.digested);
    }

    res
  }

  pub fn lex_all(&mut self) -> LexAllOutput<Rc<Token<'buffer, Kind, ErrorType>>> {
    let mut output = LexAllOutput {
      tokens: Vec::new(),
      digested: 0,
      errors: Vec::new(),
    };

    loop {
      let res = self.lex();

      output.digested += res.digested;
      output.errors.extend(res.errors);

      if let Some(token) = res.token {
        output.tokens.push(token);
      } else {
        return output;
      }
    }
  }

  pub fn trim(&mut self) -> TrimOutput<Rc<Token<'buffer, Kind, ErrorType>>> {
    // if already trimmed, return empty output
    if self.state.trimmed() {
      return TrimOutput {
        digested: 0,
        errors: Vec::new(),
      };
    }

    let res = self.core.trim(self.state.buffer(), self.state.digested());
    self.state.trim(res.digested);
    res
  }
}
