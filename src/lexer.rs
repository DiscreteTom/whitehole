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
  token::{Token, TokenKind},
};
use std::rc::Rc;

#[derive(Clone)]
pub struct Lexer<Kind: 'static, ActionState: 'static, ErrorType: 'static> {
  core: LexerCore<Kind, ActionState, ErrorType>,
  buffer: String,
  digested: usize,
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static> Lexer<Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone,
{
  pub fn new(actions: Vec<Action<Kind, ActionState, ErrorType>>, state: ActionState) -> Self {
    Lexer {
      core: LexerCore::new(actions, state),
      buffer: String::new(),
      digested: 0,
    }
  }

  pub fn reset(&mut self) -> &mut Self {
    self.core.reset();
    self.buffer.clear();
    self.digested = 0;
    self
  }

  pub fn feed(&mut self, s: &str) -> &mut Self {
    self.buffer.push_str(s);
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
      self.buffer.as_str(),
      LexerCoreLexOptions {
        start: self.digested,
        peek: options.peek,
        expectation: options.expectation,
      },
    );

    // update state if not peek
    // TODO
    // if (!options.peek) {
    // }

    res.token
  }
}
