pub mod action;
pub mod builder;
pub mod core;
pub mod position;
pub mod state;
pub mod token;
pub mod trimmed;

pub use action::Action;
pub use builder::Builder;

use self::{
  core::{
    lex::{
      expectation::Expectation, options::LexerCoreLexOptions, LexAllOutput, LexOutput, PeekOutput,
    },
    trim::{IntoTrimmedOutput, TrimOutput},
    LexerCore,
  },
  state::LexerState,
  token::{Token, TokenKind},
  trimmed::TrimmedLexer,
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

  pub fn core(&self) -> &LexerCore<Kind, ActionState, ErrorType> {
    &self.core
  }
  pub fn core_mut(&mut self) -> &mut LexerCore<Kind, ActionState, ErrorType> {
    &mut self.core
  }
  pub fn state(&self) -> &LexerState<'buffer> {
    &self.state
  }

  // TODO: better name?
  pub fn dry_clone<'new_buffer>(
    &self,
    buffer: &'new_buffer str,
  ) -> Lexer<'new_buffer, Kind, ActionState, ErrorType> {
    Lexer {
      core: self.core.dry_clone(),
      state: LexerState::new(buffer),
    }
  }

  pub fn rest(&self) -> &'buffer str {
    &self.state.buffer()[self.state.digested()..]
  }

  /// Peek the next token without updating the state.
  /// This will clone the ActionState and return it.
  pub fn peek(&self) -> PeekOutput<Rc<Token<'buffer, Kind, ErrorType>>, ActionState> {
    self.peek_expect(Expectation::default())
  }

  /// Peek the next token without updating the state.
  /// This will clone the ActionState and return it.
  pub fn peek_expect<'expect_text>(
    &self,
    expectation: impl Into<Expectation<'expect_text, Kind>>,
  ) -> PeekOutput<Rc<Token<'buffer, Kind, ErrorType>>, ActionState> {
    let mut core = self.core.clone();
    let output = core.lex(
      self.state.buffer(),
      LexerCoreLexOptions {
        start: self.state.digested(),
        expectation: expectation.into(),
      },
    );
    PeekOutput {
      token: output.token,
      digested: output.digested,
      errors: output.errors,
      state: core.take_state(),
    }
  }

  pub fn lex(&mut self) -> LexOutput<Rc<Token<'buffer, Kind, ErrorType>>> {
    self.lex_expect(Expectation::default())
  }

  pub fn lex_expect<'expect_text>(
    &mut self,
    expectation: impl Into<Expectation<'expect_text, Kind>>,
  ) -> LexOutput<Rc<Token<'buffer, Kind, ErrorType>>> {
    let res = self.core.lex(
      self.state.buffer(),
      LexerCoreLexOptions {
        start: self.state.digested(),
        expectation: expectation.into(),
      },
    );

    // update state if not peek
    self.state.digest(res.digested);

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

  /// Digest the next n chars and set the action state.
  /// If the `state` is not provided, the action state will be reset to default.
  pub fn take(&mut self, n: usize, state: Option<ActionState>) -> &mut Self {
    self.state.digest(n);
    *self.core.state_mut() = state.unwrap_or(ActionState::default());
    self
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

  pub fn into_trimmed(
    mut self,
  ) -> IntoTrimmedOutput<
    Rc<Token<'buffer, Kind, ErrorType>>,
    TrimmedLexer<'buffer, Kind, ActionState, ErrorType>,
  > {
    let res = self.trim();
    IntoTrimmedOutput {
      digested: res.digested,
      errors: res.errors,
      trimmed: TrimmedLexer::new(self),
    }
  }
}
