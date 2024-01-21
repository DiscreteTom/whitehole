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
pub struct Lexer<'buffer, Kind: 'static, ActionState: 'static, ErrorType: 'static> {
  core: LexerCore<Kind, ActionState, ErrorType>,
  state: LexerState<'buffer>,
}

impl<'buffer, Kind: 'static, ActionState: 'static, ErrorType: 'static>
  Lexer<'buffer, Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone,
{
  pub fn new(
    actions: Vec<Action<Kind, ActionState, ErrorType>>,
    state: ActionState,
    buffer: &'buffer str,
  ) -> Self {
    Lexer {
      core: LexerCore::new(actions, state),
      state: LexerState::new(buffer),
    }
  }

  pub fn lex(&mut self) -> Option<Rc<Token<Kind, ErrorType>>> {
    self.lex_with(LexerLexOptions::default())
  }

  pub fn lex_with<'input_text, 'expect>(
    &mut self,
    options: impl Into<LexerLexOptions<'input_text, 'expect, Kind>>,
  ) -> Option<Rc<Token<Kind, ErrorType>>> {
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

    res.token
  }

  pub fn rest(&self) -> &str {
    &self.state.buffer()[self.state.digested()..]
  }
}
