use super::{
  options::{LexOptions, ReLexContext},
  output::LexOutput,
  state::LexerState,
  stateless::{StatelessLexOptions, StatelessLexer},
  token::{Token, TokenKindIdProvider},
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

  /// Peek the next token without updating the state.
  /// This will clone the [`Self::action_state`] and return it.
  pub fn peek(
    &self,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexContext>,
    ActionState,
  )
  where
    Kind: TokenKindIdProvider<Kind>,
    ActionState: Clone,
  {
    self.peek_with_options(LexOptions::default())
  }

  /// Peek the next token with custom options, without updating [`Self::state`] and [`Self::action_state`].
  /// This will clone the [`Self::action_state`] and return it.
  pub fn peek_with<'expect_text>(
    &self,
    options_builder: impl FnOnce(LexOptions<'expect_text, Kind>) -> LexOptions<'expect_text, Kind>,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexContext>,
    ActionState,
  )
  where
    Kind: TokenKindIdProvider<Kind>,
    ActionState: Clone,
  {
    self.peek_with_options(options_builder(LexOptions::default()))
  }

  /// Peek the next token with custom options, without updating [`Self::state`] and [`Self::action_state`].
  /// This will clone the [`Self::action_state`] and return it.
  pub fn peek_with_options<'expect_text>(
    &self,
    options: impl Into<LexOptions<'expect_text, Kind>>,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexContext>,
    ActionState,
  )
  where
    Kind: TokenKindIdProvider<Kind>,
    ActionState: Clone,
  {
    let options: LexOptions<_> = options.into();

    // because of peek, clone the action state to prevent mutating the original one
    let mut tmp_action_state = self.action_state.clone();

    let output = Self::lex_with_stateless(
      &self.stateless,
      &self.state,
      &mut tmp_action_state, // use the cloned action state
      options,
    );

    // don't update lexer state

    (output, tmp_action_state)
  }

  /// Try to yield the next token.
  /// [`Self::state`] and [`Self::action_state`] will be updated.
  pub fn lex(&mut self) -> LexOutput<Token<'text, Kind, ErrorType>, ReLexContext>
  where
    Kind: TokenKindIdProvider<Kind>,
  {
    self.lex_with_options(LexOptions::default())
  }

  /// Try to yield the next token with custom options.
  /// [`Self::state`] and [`Self::action_state`] will be updated.
  pub fn lex_with<'expect_text>(
    &mut self,
    options_builder: impl FnOnce(LexOptions<'expect_text, Kind>) -> LexOptions<'expect_text, Kind>,
  ) -> LexOutput<Token<'text, Kind, ErrorType>, ReLexContext>
  where
    Kind: TokenKindIdProvider<Kind>,
  {
    self.lex_with_options(options_builder(LexOptions::default()))
  }

  /// Try to yield the next token with custom options.
  /// [`Self::state`] and [`Self::action_state`] will be updated.
  pub fn lex_with_options<'expect_text>(
    &mut self,
    options: impl Into<LexOptions<'expect_text, Kind>>,
  ) -> LexOutput<Token<'text, Kind, ErrorType>, ReLexContext>
  where
    Kind: TokenKindIdProvider<Kind>,
  {
    let options: LexOptions<_> = options.into();

    let output = Self::lex_with_stateless(
      &self.stateless,
      &self.state,
      &mut self.action_state,
      options,
    );

    // update state
    self.state.digest(output.digested);

    output
  }

  /// Digest the next (at most) n chars and set the [`Self::action_state`].
  pub fn take_with(&mut self, n: usize, action_state: ActionState) -> &mut Self {
    self.state.digest(n); // TODO: validate n to prevent too big
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
    stateless: &Rc<StatelessLexer<Kind, ActionState, ErrorType>>,
    state: &LexerState<'text>,
    action_state: &mut ActionState,
    options: LexOptions<'expect_text, Kind>,
  ) -> LexOutput<Token<'text, Kind, ErrorType>, ReLexContext>
  where
    Kind: TokenKindIdProvider<Kind>,
  {
    stateless.lex_with_options(
      state.text(),
      action_state,
      StatelessLexOptions {
        start: state.digested(),
        base: options,
      },
    )
  }
}

impl<Kind, ActionState, ErrorType> StatelessLexer<Kind, ActionState, ErrorType> {
  /// Consume self, create a new lexer with the provided text.
  pub fn into_lexer(self, text: &str) -> Lexer<Kind, ActionState, ErrorType>
  where
    ActionState: Default,
  {
    self.into_lexer_with(ActionState::default(), text)
  }

  /// Consume self, create a new lexer with the provided action state and text.
  pub fn into_lexer_with(
    self,
    action_state: ActionState,
    text: &str,
  ) -> Lexer<Kind, ActionState, ErrorType> {
    Lexer::new(Rc::new(self), action_state, text)
  }
}
