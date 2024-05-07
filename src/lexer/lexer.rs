use super::{
  fork::{ForkDisabled, LexOptionsFork},
  options::LexOptions,
  output::LexOutput,
  state::LexerState,
  stateless::{StatelessLexOptions, StatelessLexer},
  token::{Token, TokenKindIdProvider},
};
use std::{cmp::min, rc::Rc};

pub struct Lexer<'text, Kind: 'static, ActionState, ErrorType> {
  // use Rc so that this is clone-able
  stateless: Rc<StatelessLexer<Kind, ActionState, ErrorType>>,
  state: LexerState<'text>,
  // user can mutate the action state directly, so just make it public
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

  /// Clone self with a new action state.
  pub fn clone_with(&self, action_state: ActionState) -> Self {
    Self {
      stateless: self.stateless.clone(),
      state: self.state.clone(),
      action_state,
    }
  }

  /// Consume self, return a new lexer with the same actions and a new text.
  /// [`Self::state`] and [`Self::action_state`] will be reset to default.
  pub fn reload<'new_text>(
    self,
    text: &'new_text str,
  ) -> Lexer<'new_text, Kind, ActionState, ErrorType>
  where
    ActionState: Default,
  {
    Lexer {
      stateless: self.stateless,
      state: LexerState::new(text),
      action_state: ActionState::default(),
    }
  }

  /// Consume self, return a new lexer with the same actions, a new text and the given action state.
  /// [`Self::state`] will be reset to default.
  pub fn reload_with<'new_text>(
    self,
    text: &'new_text str,
    action_state: ActionState,
  ) -> Lexer<'new_text, Kind, ActionState, ErrorType> {
    Lexer {
      stateless: self.stateless,
      state: LexerState::new(text),
      action_state,
    }
  }

  /// Peek the next token with the default options, without updating
  /// [`Self::state`] and [`Self::action_state`].
  /// This will clone [`Self::action_state`] and return it.
  pub fn peek(&self) -> (LexOutput<Token<'text, Kind, ErrorType>, ()>, ActionState)
  where
    Kind: TokenKindIdProvider<Kind>,
    ActionState: Clone,
  {
    self.peek_with_options(LexOptions::default())
  }

  /// Peek the next token with custom options, without updating
  /// [`Self::state`] and [`Self::action_state`].
  /// This will clone the [`Self::action_state`] by default and return it.
  /// If this is a re-lex, [`ReLexable::action_state`](crate::lexer::options::ReLexable::action_state)
  /// will be used instead of cloning [`Self::action_state`].
  pub fn peek_with<'expect_text, Fork: LexOptionsFork<'text, 'expect_text, Kind, ActionState>>(
    &self,
    options_builder: impl FnOnce(
      LexOptions<'expect_text, Kind, ForkDisabled>,
    ) -> LexOptions<'expect_text, Kind, Fork>,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, Fork::ReLexableType>,
    ActionState,
  )
  where
    Kind: TokenKindIdProvider<Kind>,
    ActionState: Clone,
  {
    self.peek_with_options(options_builder(LexOptions::default()))
  }

  /// Peek the next token with custom options, without updating
  /// [`Self::state`] and [`Self::action_state`].
  /// This will clone the [`Self::action_state`] by default and return it.
  /// If this is a re-lex, [`ReLexable::action_state`](crate::lexer::options::ReLexable::action_state)
  /// will be used instead of cloning [`Self::action_state`].
  pub fn peek_with_options<
    'expect_text,
    Fork: LexOptionsFork<'text, 'expect_text, Kind, ActionState>,
  >(
    &self,
    options: impl Into<LexOptions<'expect_text, Kind, Fork>>,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, Fork::ReLexableType>,
    ActionState,
  )
  where
    Kind: TokenKindIdProvider<Kind>,
    ActionState: Clone,
  {
    let options: LexOptions<_, _> = options.into();

    // TODO: replace with a method `re_lex`
    // because of peek, we won't mutate lexer's action state.
    // if this is a re-lex and user provides action state, use it,
    // otherwise clone the action state to prevent mutating the original one
    // let mut tmp_action_state = options
    //   .re_lex
    //   .as_mut()
    //   .and_then(|re_lex| re_lex.action_state.take())
    //   .unwrap_or_else(|| self.action_state.clone());
    let mut tmp_action_state = self.action_state.clone();

    let output = Self::lex_with_stateless(
      &self.stateless,
      &self.state,
      &mut tmp_action_state, // don't use self.action_state
      options,
    );

    // don't update lexer state

    (output, tmp_action_state)
  }

  /// Try to yield the next token with the default options.
  /// [`Self::state`] and [`Self::action_state`] will be updated.
  pub fn lex(&mut self) -> LexOutput<Token<'text, Kind, ErrorType>, ()>
  where
    Kind: TokenKindIdProvider<Kind>,
  {
    self.lex_with_options(LexOptions::default())
  }

  /// Try to yield the next token with custom options.
  /// [`Self::state`] and [`Self::action_state`] will be updated.
  pub fn lex_with<'expect_text, Fork: LexOptionsFork<'text, 'expect_text, Kind, ActionState>>(
    &mut self,
    options_builder: impl FnOnce(
      LexOptions<'expect_text, Kind, ForkDisabled>,
    ) -> LexOptions<'expect_text, Kind, Fork>,
  ) -> LexOutput<Token<'text, Kind, ErrorType>, Fork::ReLexableType>
  where
    Kind: TokenKindIdProvider<Kind>,
  {
    self.lex_with_options(options_builder(LexOptions::default()))
  }

  /// Try to yield the next token with custom options.
  /// [`Self::state`] and [`Self::action_state`] will be updated.
  pub fn lex_with_options<
    'expect_text,
    Fork: LexOptionsFork<'text, 'expect_text, Kind, ActionState>,
  >(
    &mut self,
    options: impl Into<LexOptions<'expect_text, Kind, Fork>>,
  ) -> LexOutput<Token<'text, Kind, ErrorType>, Fork::ReLexableType>
  where
    Kind: TokenKindIdProvider<Kind>,
  {
    let options: LexOptions<_, _> = options.into();

    // TODO: replace with a method `re_lex`
    // if this is a re-lex and action state is provided, set it
    // options.re_lex.as_mut().map(|re_lex| {
    //   re_lex.action_state.take().map(|action_state| {
    //     self.action_state = action_state;
    //   })
    // });

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

  /// Digest the next (at most) `n` chars and set [`Self::action_state`].
  pub fn digest_and_set_action_state(&mut self, n: usize, action_state: ActionState) -> &mut Self {
    self
      .state
      .digest(min(n, self.state.text().len() - self.state.digested()));
    self.action_state = action_state;
    self
  }

  /// Digest the next (at most) `n` chars and set [`Self::action_state`] to default.
  pub fn digest(&mut self, n: usize) -> &mut Self
  where
    ActionState: Default,
  {
    self.digest_and_set_action_state(n, ActionState::default())
  }

  fn lex_with_stateless<
    'expect_text,
    Fork: LexOptionsFork<'text, 'expect_text, Kind, ActionState>,
  >(
    stateless: &Rc<StatelessLexer<Kind, ActionState, ErrorType>>,
    state: &LexerState<'text>,
    action_state: &mut ActionState,
    options: LexOptions<'expect_text, Kind, Fork>,
  ) -> LexOutput<Token<'text, Kind, ErrorType>, Fork::ReLexableType>
  where
    Kind: TokenKindIdProvider<Kind>,
  {
    stateless.lex_with_options(
      state.text(),
      StatelessLexOptions {
        start: state.digested(),
        action_state,
        base: options,
      },
    )
  }
}
