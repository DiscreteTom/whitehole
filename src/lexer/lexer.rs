use super::{
  expectation::Expectation,
  options::{LexOptions, ReLexContext},
  output::{LexAllOutput, LexOutput, ReLexable, TrimOutput},
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
  // user can mutate the action state
  pub action_state: ActionState,
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
  pub fn peek(
    &self,
  ) -> (
    LexOutput<Token<'buffer, Kind, ErrorType>, ReLexable<()>>,
    ActionState,
  ) {
    self.peek_with(LexOptions::default())
  }

  /// Peek the next token with expectation, without updating the state.
  /// This will clone the ActionState and return it.
  pub fn peek_expect<'expect_text>(
    &self,
    expectation: impl Into<Expectation<'expect_text, Kind>>,
  ) -> (
    LexOutput<Token<'buffer, Kind, ErrorType>, ReLexable<()>>,
    ActionState,
  ) {
    self.peek_with(LexOptions::default().expect(expectation))
  }

  /// Peek the next token and return the result with re-lex context, without updating the state.
  /// This will clone the ActionState and return it.
  pub fn peek_fork(
    &self,
  ) -> (
    LexOutput<Token<'buffer, Kind, ErrorType>, ReLexable<()>>,
    ActionState,
  ) {
    self.peek_with(LexOptions::default().fork())
  }

  pub fn peek_with<'expect_text>(
    &self,
    options: impl Into<LexOptions<'expect_text, Kind>>,
  ) -> (
    LexOutput<Token<'buffer, Kind, ErrorType>, ReLexable<()>>,
    ActionState,
  ) {
    let options = options.into() as LexOptions<_>;

    // because of peek, clone the action state to prevent mutation
    let mut tmp_action_state = self.action_state.clone();

    let output = if options.fork {
      let res =
        self.peek_with_stateless(&mut tmp_action_state, options.expectation, options.re_lex);
      LexOutput {
        token: res.token,
        digested: res.digested,
        errors: res.errors,
        re_lex: res.re_lex.map(|context| {
          ReLexable {
            // since self is not mutated, we don't need to clone it
            // nor construct a new lexer
            lexer: (),
            context,
          }
        }),
      }
    } else {
      let res =
        self.peek_with_stateless(&mut tmp_action_state, options.expectation, options.re_lex);
      LexOutput {
        token: res.token,
        digested: res.digested,
        errors: res.errors,
        re_lex: None, // fork is not enabled, so re_lex is not available
      }
    };

    // don't update lexer state

    (output, tmp_action_state)
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

    let output = if options.fork {
      // fork is enabled, backup the action state before changing it
      let action_state_bk = self.action_state.clone();
      let res = self.lex_with_stateless(options.expectation, options.re_lex);
      LexOutput {
        token: res.token,
        digested: res.digested,
        errors: res.errors,
        re_lex: res.re_lex.map(|context| {
          ReLexable {
            // construct a lexer with the state before lex
            lexer: Self {
              stateless: self.stateless.clone(),
              state: self.state.clone(), // self.state is not mutated yet
              action_state: action_state_bk,
            },
            context,
          }
        }),
      }
    } else {
      let res = self.lex_with_stateless(options.expectation, options.re_lex);
      LexOutput {
        token: res.token,
        digested: res.digested,
        errors: res.errors,
        re_lex: None, // fork is not enabled, so re_lex is not available
      }
    };

    // update state
    self.state.digest(output.digested);

    output
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
  pub fn take(&mut self, n: usize, state: impl Into<Option<ActionState>>) -> &mut Self {
    self.state.digest(n);
    self.action_state = state.into().unwrap_or(ActionState::default());
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

  fn lex_with_stateless<'expect_text>(
    &mut self,
    expectation: Expectation<'expect_text, Kind>,
    re_lex: Option<ReLexContext>,
  ) -> LexOutput<Token<'buffer, Kind, ErrorType>, ReLexContext> {
    self.stateless.lex_with(
      self.state.buffer(),
      StatelessLexOptions {
        start: self.state.digested(),
        action_state: &mut self.action_state,
        expectation,
        re_lex,
      },
    )
  }

  // TODO: merge duplicated code
  fn peek_with_stateless<'expect_text>(
    &self,
    action_state: &mut ActionState,
    expectation: Expectation<'expect_text, Kind>,
    re_lex: Option<ReLexContext>,
  ) -> LexOutput<Token<'buffer, Kind, ErrorType>, ReLexContext> {
    self.stateless.lex_with(
      self.state.buffer(),
      StatelessLexOptions {
        start: self.state.digested(),
        action_state,
        expectation,
        re_lex,
      },
    )
  }
}
