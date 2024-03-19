use super::{
  expectation::Expectation,
  options::{LexOptions, ReLexContext},
  output::{LexAllOutput, LexOutput, ReLexable},
  state::LexerState,
  stateless::{StatelessLexOptions, StatelessLexer, StatelessReLexable},
  token::{Token, TokenKind},
};
use std::rc::Rc;

// TODO: impl iterator?
pub struct Lexer<'text, Kind, ActionState, ErrorType> {
  // use Rc so that this is clone-able
  stateless: Rc<StatelessLexer<Kind, ActionState, ErrorType>>,
  state: LexerState<'text>,
  // user can mutate the action state
  pub action_state: ActionState,
}

impl<'text, Kind, ActionState, ErrorType> Clone for Lexer<'text, Kind, ActionState, ErrorType>
where
  ActionState: Clone,
{
  fn clone(&self) -> Self {
    Self {
      stateless: self.stateless.clone(),
      state: self.state.clone(),
      action_state: self.action_state.clone(),
    }
  }
}

impl<'text, Kind, ActionState, ErrorType> Lexer<'text, Kind, ActionState, ErrorType> {
  pub fn new(
    stateless: Rc<StatelessLexer<Kind, ActionState, ErrorType>>,
    action_state: ActionState,
    text: &'text str,
  ) -> Self {
    Self {
      stateless,
      state: LexerState::new(text),
      action_state,
    }
  }

  pub fn with_default_action_state(
    stateless: Rc<StatelessLexer<Kind, ActionState, ErrorType>>,
    text: &'text str,
  ) -> Self
  where
    ActionState: Default,
  {
    Self::new(stateless, ActionState::default(), text)
  }

  pub fn stateless(&self) -> &Rc<StatelessLexer<Kind, ActionState, ErrorType>> {
    &self.stateless
  }
  // user is not able to mutate the lexer state directly
  pub fn state(&self) -> &LexerState<'text> {
    &self.state
  }

  /// Consume self, return a new lexer with the same actions
  /// and the provided text and action state.
  /// The [`Self::state`] will be reset to default.
  pub fn reload_with<'new_text>(
    self,
    action_state: ActionState,
    text: &'new_text str,
  ) -> Lexer<'new_text, Kind, ActionState, ErrorType> {
    Lexer {
      stateless: self.stateless,
      state: LexerState::new(text),
      action_state,
    }
  }

  /// Consume self, return a new lexer with the same actions and a new text.
  /// The [`Self::state`] and [`Self::action_state`] will be reset to default.
  pub fn reload<'new_text>(
    self,
    text: &'new_text str,
  ) -> Lexer<'new_text, Kind, ActionState, ErrorType>
  where
    ActionState: Default,
  {
    self.reload_with(ActionState::default(), text)
  }

  /// Clone the lexer and load a new input text and action state.
  /// The [`Self::state`] will be reset to default.
  pub fn clone_with<'new_text>(
    &self,
    action_state: ActionState,
    text: &'new_text str,
  ) -> Lexer<'new_text, Kind, ActionState, ErrorType> {
    Lexer {
      stateless: self.stateless.clone(),
      state: LexerState::new(text),
      action_state,
    }
  }

  /// Clone the lexer and load a new input text.
  /// The [`Self::state`] and [`Self::action_state`] will be reset to default.
  pub fn clone_with_default_action_state<'new_text>(
    &self,
    text: &'new_text str,
  ) -> Lexer<'new_text, Kind, ActionState, ErrorType>
  where
    ActionState: Default,
  {
    Lexer {
      stateless: self.stateless.clone(),
      state: LexerState::new(text),
      action_state: ActionState::default(),
    }
  }

  /// Peek the next token without updating the state.
  /// This will clone the [`Self::action_state`] and return it.
  pub fn peek(
    &self,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexable<()>>,
    ActionState,
  )
  where
    Kind: TokenKind<Kind> + 'static,
    ActionState: Clone,
  {
    self.peek_with(LexOptions::default())
  }

  /// Peek the next token with expectation, without updating [`Self::state`] and [`Self::action_state`].
  /// This will clone the [`Self::action_state`] and return it.
  pub fn peek_expect<'expect_text>(
    &self,
    expectation: impl Into<Expectation<'expect_text, Kind>>,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexable<()>>,
    ActionState,
  )
  where
    Kind: TokenKind<Kind> + 'static,
    ActionState: Clone,
  {
    self.peek_with(LexOptions::default().expect(expectation))
  }

  /// Peek the next token without updating [`Self::state`] and [`Self::action_state`].
  /// If the lex is re-lex-able, the [`LexOutput::re_lex`] will be `Some`.
  /// This will clone the [`Self::action_state`] and return it.
  pub fn peek_fork(
    &self,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexable<()>>,
    ActionState,
  )
  where
    Kind: TokenKind<Kind> + 'static,
    ActionState: Clone,
  {
    self.peek_with(LexOptions::default().fork())
  }

  /// Peek the next token with custom options, without updating [`Self::state`] and [`Self::action_state`].
  /// This will clone the [`Self::action_state`] and return it.
  pub fn peek_with<'expect_text>(
    &self,
    options: impl Into<LexOptions<'expect_text, Kind>>,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexable<()>>,
    ActionState,
  )
  where
    Kind: TokenKind<Kind> + 'static,
    ActionState: Clone,
  {
    let options = options.into() as LexOptions<_>;

    // TODO: prevent clone here?
    // because of peek, clone the action state to prevent mutation
    let mut tmp_action_state = self.action_state.clone();

    // TODO: optimize code, push down fork logic to stateless?
    let output = if options.fork {
      let res =
        self.peek_with_stateless(&mut tmp_action_state, options.expectation, options.re_lex);
      LexOutput {
        token: res.token,
        digested: res.digested,
        errors: res.errors,
        re_lex: res.re_lex.map(|re_lexable| {
          ReLexable {
            // since self is not mutated, we don't need to clone it
            // nor construct a new lexer
            lexer: (),
            context: re_lexable.context,
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

  /// Try to yield the next token.
  /// [`Self::state`] and [`Self::action_state`] will be updated.
  pub fn lex(&mut self) -> LexOutput<Token<'text, Kind, ErrorType>, ReLexable<Self>>
  where
    Kind: TokenKind<Kind> + 'static,
    ActionState: Clone,
  {
    self.lex_with(LexOptions::default())
  }

  /// Try to yield the next token with expectation.
  /// [`Self::state`] and [`Self::action_state`] will be updated.
  pub fn lex_expect<'expect_text>(
    &mut self,
    expectation: impl Into<Expectation<'expect_text, Kind>>,
  ) -> LexOutput<Token<'text, Kind, ErrorType>, ReLexable<Self>>
  where
    Kind: TokenKind<Kind> + 'static,
    ActionState: Clone,
  {
    self.lex_with(LexOptions::default().expect(expectation))
  }

  /// Try to yield the next token.
  /// If the lex is re-lex-able, the [`LexOutput::re_lex`] will be `Some`.
  /// [`Self::state`] and [`Self::action_state`] will be updated.
  pub fn lex_fork(&mut self) -> LexOutput<Token<'text, Kind, ErrorType>, ReLexable<Self>>
  where
    Kind: TokenKind<Kind> + 'static,
    ActionState: Clone,
  {
    self.lex_with(LexOptions::default().fork())
  }

  /// Try to yield the next token with custom options.
  /// [`Self::state`] and [`Self::action_state`] will be updated.
  pub fn lex_with<'expect_text>(
    &mut self,
    options: impl Into<LexOptions<'expect_text, Kind>>,
  ) -> LexOutput<Token<'text, Kind, ErrorType>, ReLexable<Self>>
  where
    Kind: TokenKind<Kind> + 'static,
    ActionState: Clone, // TODO: lex shouldn't require Clone, maybe add new method `lex_fork_with`
  {
    let options = options.into() as LexOptions<_>;

    let output = self.lex_with_stateless(options.expectation, options.re_lex);
    let res = LexOutput {
      token: output.token,
      digested: output.digested,
      errors: output.errors,
      re_lex: output.re_lex.map(|re_lexable| {
        ReLexable {
          // construct a lexer with the state before lex
          lexer: Self {
            stateless: self.stateless.clone(),
            state: self.state.clone(), // self.state is not mutated yet
            action_state: re_lexable.state,
          },
          context: re_lexable.context,
        }
      }),
    };

    // update state
    self.state.digest(output.digested);

    res
  }

  // TODO: add options?
  pub fn lex_all(&mut self) -> LexAllOutput<Token<'text, Kind, ErrorType>>
  where
    Kind: TokenKind<Kind> + 'static,
    ActionState: Clone,
  {
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

  /// Digest the next (at most) n chars and set the [`Self::action_state`].
  pub fn take_with(&mut self, n: usize, action_state: ActionState) -> &mut Self {
    self.state.digest(n); // TODO: validate n
    self.action_state = action_state;
    self
  }

  /// Digest the next (at most) n chars and set the [`Self::action_state`] to default.
  pub fn take(&mut self, n: usize) -> &mut Self
  where
    ActionState: Default,
  {
    self.take_with(n, ActionState::default())
  }

  fn lex_with_stateless<'expect_text>(
    &mut self,
    expectation: Expectation<'expect_text, Kind>,
    re_lex: Option<ReLexContext>,
  ) -> LexOutput<Token<'text, Kind, ErrorType>, StatelessReLexable<ActionState>>
  where
    Kind: TokenKind<Kind>,
    ActionState: Clone,
  {
    self.stateless.lex_with_options(
      self.state.text(),
      &mut self.action_state,
      StatelessLexOptions {
        start: self.state.digested(),
        base: LexOptions {
          expectation,
          fork: false, // TODO: directly use LexOptions
          re_lex,
        },
      },
    )
  }

  // TODO: merge duplicated code
  fn peek_with_stateless<'expect_text>(
    &self,
    action_state: &mut ActionState,
    expectation: Expectation<'expect_text, Kind>,
    re_lex: Option<ReLexContext>,
  ) -> LexOutput<Token<'text, Kind, ErrorType>, StatelessReLexable<ActionState>>
  where
    Kind: TokenKind<Kind>,
    ActionState: Clone,
  {
    self.stateless.lex_with_options(
      self.state.text(),
      action_state,
      StatelessLexOptions {
        start: self.state.digested(),
        base: LexOptions {
          expectation,
          fork: false, // TODO: directly use LexOptions
          re_lex,
        },
      },
    )
  }
}

impl<Kind, ActionState, ErrorType> StatelessLexer<Kind, ActionState, ErrorType> {
  /// Consume self, create a new lexer with the provided text.
  pub fn into_lexer(self, text: &str) -> Lexer<Kind, ActionState, ErrorType>
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    ActionState: Default, // TODO: add a function that accept an action state instead of default
  {
    Lexer::with_default_action_state(Rc::new(self), text)
  }
}
