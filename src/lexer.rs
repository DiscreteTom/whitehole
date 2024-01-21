pub mod action;
pub mod builder;
pub mod core;
pub mod options;
pub mod state;
pub mod token;

use self::{
  action::Action,
  core::{lex::options::LexerCoreLexOptions, LexerCore},
  options::LexerLexOptions,
  state::LexerState,
  token::{Token, TokenKind},
};
use std::rc::Rc;

#[derive(Clone)]
pub struct Lexer<Kind: 'static, ActionState: 'static, ErrorType: 'static> {
  buffer: String,
  core: LexerCore<Kind, ActionState, ErrorType>,
  state: LexerState,
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static> Lexer<Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone,
{
  pub fn new(actions: Vec<Action<Kind, ActionState, ErrorType>>, state: ActionState) -> Self {
    Lexer {
      buffer: String::default(),
      core: LexerCore::new(actions, state),
      state: LexerState::default(),
    }
  }

  pub fn reset(&mut self) -> &mut Self {
    self.buffer.clear();
    self.core.reset();
    self.state.reset();
    self
  }

  pub fn feed(&mut self, s: &str) -> &mut Self {
    self.buffer += s;
    self.state.on_feed();
    self
  }

  pub fn lex(&mut self) -> Option<Rc<Token<Kind, ErrorType>>> {
    self.lex_with(LexerLexOptions::default())
  }

  pub fn lex_with<'input_text, 'expect>(
    &mut self,
    options: impl Into<LexerLexOptions<'input_text, 'expect, Kind>>,
  ) -> Option<Rc<Token<Kind, ErrorType>>> {
    let options: LexerLexOptions<Kind> = options.into();

    // feed input if any
    if let Some(input) = options.input {
      self.feed(input);
    }

    let res = self.core.lex(
      &self.buffer,
      LexerCoreLexOptions {
        start: self.state.digested(),
        peek: options.peek,
        expectation: options.expectation,
      },
    );

    // update state if not peek
    if !options.peek {
      self.state.on_digest(res.digested, &self.buffer);
    }

    res.token
  }

  pub fn rest(&self) -> &str {
    &self.buffer[self.state.digested()..]
  }
}
