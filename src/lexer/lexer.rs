use super::{
  expectation::Expectation,
  options::LexOptions,
  output::{LexAllOutput, LexOutput, PeekOutput, ReLexable, TrimOutput},
  state::LexerState,
  stateless::{lex::StatelessLexOptions, StatelessLexer},
  token::{Token, TokenKind},
};
use std::rc::Rc;

pub struct Lexer<'buffer, Kind: 'static, ActionState: 'static, ErrorType: 'static>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  // use Rc so that this is clone-able
  stateless: Rc<StatelessLexer<Kind, ActionState, ErrorType>>,
  state: LexerState<'buffer>,
  action_state: ActionState,
}

impl<'buffer, Kind: 'static, ActionState: 'static, ErrorType: 'static> Clone
  for Lexer<'buffer, Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  fn clone(&self) -> Self {
    Lexer {
      stateless: self.stateless.clone(),
      state: self.state.clone(),
      action_state: self.action_state.clone(),
    }
  }
}

impl<'buffer, Kind: 'static, ActionState: 'static, ErrorType: 'static>
  Lexer<'buffer, Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  pub fn new(
    stateless: Rc<StatelessLexer<Kind, ActionState, ErrorType>>,
    buffer: &'buffer str,
  ) -> Self {
    Lexer {
      stateless,
      state: LexerState::new(buffer),
      action_state: ActionState::default(),
    }
  }

  pub fn stateless(&self) -> &Rc<StatelessLexer<Kind, ActionState, ErrorType>> {
    &self.stateless
  }
  // user is not able to mutate the lexer state directly
  pub fn state(&self) -> &LexerState<'buffer> {
    &self.state
  }
  pub fn action_state(&self) -> &ActionState {
    &self.action_state
  }
  // user can mutate the action state
  pub fn action_state_mut(&mut self) -> &mut ActionState {
    &mut self.action_state
  }

  /// Consume self, return a new lexer with the same actions and a new buffer.
  /// LexerState and ActionState will be reset to default.
  pub fn reload<'new_buffer>(
    self,
    buffer: &'new_buffer str,
  ) -> Lexer<'new_buffer, Kind, ActionState, ErrorType> {
    Lexer {
      stateless: self.stateless,
      state: LexerState::new(buffer),
      action_state: ActionState::default(),
    }
  }

  /// Clone the lexer and load a new input text.
  /// LexerState and ActionState will be reset to default.
  pub fn clone_with<'new_buffer>(
    &self,
    buffer: &'new_buffer str,
  ) -> Lexer<'new_buffer, Kind, ActionState, ErrorType> {
    Lexer {
      stateless: self.stateless.clone(),
      state: LexerState::new(buffer),
      action_state: ActionState::default(),
    }
  }

  /// Peek the next token without updating the state.
  /// This will clone the ActionState and return it.
  pub fn peek(&self) -> PeekOutput<Token<'buffer, Kind, ErrorType>, ActionState> {
    self.peek_expect(Expectation::default())
  }

  /// Peek the next token without updating the state.
  /// This will clone the ActionState and return it.
  pub fn peek_expect<'expect_text>(
    &self,
    expectation: impl Into<Expectation<'expect_text, Kind>>,
  ) -> PeekOutput<Token<'buffer, Kind, ErrorType>, ActionState> {
    let mut action_state = self.action_state().clone();
    let output = self.stateless.lex_with(
      self.state.buffer(),
      StatelessLexOptions {
        start: self.state.digested(),
        action_state: &mut action_state,
        expectation: expectation.into(),
        // TODO: add peek_with and make from_index configurable
        re_lex: None,
      },
    );
    PeekOutput {
      token: output.token,
      digested: output.digested,
      errors: output.errors,
      action_state,
    }
  }

  pub fn lex(&mut self) -> LexOutput<Token<'buffer, Kind, ErrorType>, ReLexable<Self>> {
    self.lex_with(LexOptions::default())
  }

  pub fn lex_expect<'expect_text>(
    &mut self,
    expectation: impl Into<Expectation<'expect_text, Kind>>,
  ) -> LexOutput<Token<'buffer, Kind, ErrorType>, ReLexable<Self>> {
    self.lex_with(LexOptions::default().expect(expectation))
  }

  pub fn lex_fork(&mut self) -> LexOutput<Token<'buffer, Kind, ErrorType>, ReLexable<Self>> {
    self.lex_with(LexOptions::default().fork())
  }

  pub fn lex_with<'expect_text>(
    &mut self,
    options: impl Into<LexOptions<'expect_text, Kind>>,
  ) -> LexOutput<Token<'buffer, Kind, ErrorType>, ReLexable<Self>> {
    let options = options.into() as LexOptions<_>;

    // if fork is enabled, backup the action state before changing it
    let action_state_bk = if options.fork {
      Some(self.action_state.clone())
    } else {
      None
    };

    let res = self.stateless.lex_with(
      self.state.buffer(),
      StatelessLexOptions {
        start: self.state.digested(),
        action_state: &mut self.action_state,
        expectation: options.expectation,
        re_lex: options.re_lex,
      },
    );

    // if fork is enabled and re-lex-able, backup the lexer state before changing it
    let state_bk = if options.fork && res.re_lex.is_some() {
      Some(self.state.clone())
    } else {
      None
    };

    // update state
    self.state.digest(res.digested);

    LexOutput {
      token: res.token,
      digested: res.digested,
      errors: res.errors,
      re_lex: if options.fork {
        res.re_lex.map(|i| ReLexable {
          context: i,
          // construct a lexer with the state before lex
          lexer: Self {
            stateless: self.stateless.clone(),
            // TODO: optimize code, prevent unwrap
            state: state_bk.unwrap(),
            action_state: action_state_bk.unwrap(),
          },
        })
      } else {
        // fork is not enabled
        None
      },
    }
  }

  pub fn lex_all(&mut self) -> LexAllOutput<Token<'buffer, Kind, ErrorType>> {
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
    self.action_state = state.unwrap_or(ActionState::default());
    self
  }

  pub fn trim(&mut self) -> TrimOutput<Token<'buffer, Kind, ErrorType>> {
    // if already trimmed, return empty output
    if self.state.trimmed() {
      return TrimOutput {
        digested: 0,
        errors: Vec::new(),
      };
    }

    let res = self.stateless.trim(
      self.state.buffer(),
      self.state.digested(),
      &mut self.action_state,
    );
    self.state.trim(res.digested);
    res
  }
}
