use super::{
  action::Accumulator,
  fork::{ForkDisabled, LexOptionsFork},
  options::LexOptions,
  output::{LexOutput, TrimOutput},
  re_lex::ReLexableFactory,
  state::LexerState,
  stateless::{StatelessLexOptions, StatelessLexer, StatelessTrimOptions},
  token::{Range, Token, TokenKindIdProvider},
};
use std::{marker::PhantomData, rc::Rc};

pub struct Lexer<'text, Kind: 'static, ActionState, ErrorType, ErrAcc> {
  // user can mutate the action state directly, so just make it public
  pub action_state: ActionState,

  // use Rc so that this is clone-able
  stateless: Rc<StatelessLexer<Kind, ActionState, ErrorType>>,
  state: LexerState<'text>,
  _err_acc: PhantomData<ErrAcc>,
}

impl<'text, Kind, ActionState, ErrorType, ErrAcc> Clone
  for Lexer<'text, Kind, ActionState, ErrorType, ErrAcc>
where
  ActionState: Clone,
{
  fn clone(&self) -> Self {
    Self {
      action_state: self.action_state.clone(),
      stateless: self.stateless.clone(),
      state: self.state.clone(),
      _err_acc: PhantomData,
    }
  }
}

impl<'text, Kind, ActionState, ErrorType, ErrAcc>
  Lexer<'text, Kind, ActionState, ErrorType, ErrAcc>
{
  pub fn new(
    stateless: Rc<StatelessLexer<Kind, ActionState, ErrorType>>,
    action_state: ActionState,
    text: &'text str,
  ) -> Self {
    Self {
      action_state,
      stateless,
      state: LexerState::new(text),
      _err_acc: PhantomData,
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
    Self::new(self.stateless.clone(), action_state, self.state.text())
  }

  /// Consume self, return a new lexer with the same actions and a new text.
  /// [`Self::state`] and [`Self::action_state`] will be reset to default.
  pub fn reload<'new_text>(
    self,
    text: &'new_text str,
  ) -> Lexer<'new_text, Kind, ActionState, ErrorType, ErrAcc>
  where
    ActionState: Default,
  {
    Lexer {
      stateless: self.stateless,
      state: LexerState::new(text),
      action_state: ActionState::default(),
      _err_acc: self._err_acc,
    }
  }

  /// Consume self, return a new lexer with the same actions, a new text and the given action state.
  /// [`Self::state`] will be reset to default.
  pub fn reload_with<'new_text>(
    self,
    action_state: ActionState,
    text: &'new_text str,
  ) -> Lexer<'new_text, Kind, ActionState, ErrorType, ErrAcc> {
    Lexer::new(self.stateless, action_state, text)
  }

  /// Peek the next token with the default options, without updating
  /// [`Self::state`] and [`Self::action_state`].
  /// This will clone [`Self::action_state`] and return it.
  pub fn peek(&self) -> (LexOutput<Token<Kind>, ErrAcc, ()>, ActionState)
  where
    Kind: TokenKindIdProvider<Kind>,
    ActionState: Clone,
    ErrAcc: Accumulator<(ErrorType, Range)> + Default,
  {
    self.peek_with_options(LexOptions::default())
  }

  /// Peek the next token with custom options, without updating
  /// [`Self::state`] and [`Self::action_state`].
  /// This will clone the [`Self::action_state`] and return it.
  pub fn peek_with<
    'expect_literal,
    Fork: LexOptionsFork<'text, Kind, ActionState, ErrorType, ErrAcc>,
  >(
    &self,
    options_builder: impl FnOnce(
      LexOptions<'expect_literal, Kind, ForkDisabled>,
    ) -> LexOptions<'expect_literal, Kind, Fork>,
  ) -> (
    LexOutput<
      Token<Kind>,
      ErrAcc,
      <Fork::ReLexableFactoryType as ReLexableFactory<
        'text,
        Kind,
        ActionState,
        ErrorType,
        ErrAcc,
      >>::ReLexableType,
    >,
    ActionState,
  )
  where
    Kind: TokenKindIdProvider<Kind>,
    ActionState: Clone,
    ErrAcc: Accumulator<(ErrorType, Range)> + Default,
  {
    self.peek_with_options(options_builder(LexOptions::default()))
  }

  /// Peek the next token with custom options, without updating
  /// [`Self::state`] and [`Self::action_state`].
  /// This will clone the [`Self::action_state`] and return it.
  pub fn peek_with_options<
    'expect_literal,
    Fork: LexOptionsFork<'text, Kind, ActionState, ErrorType, ErrAcc>,
  >(
    &self,
    options: impl Into<LexOptions<'expect_literal, Kind, Fork>>,
  ) -> (
    LexOutput<
      Token<Kind>,
      ErrAcc,
      <Fork::ReLexableFactoryType as ReLexableFactory<
        'text,
        Kind,
        ActionState,
        ErrorType,
        ErrAcc,
      >>::ReLexableType,
    >,
    ActionState,
  )
  where
    Kind: TokenKindIdProvider<Kind>,
    ActionState: Clone,
    ErrAcc: Accumulator<(ErrorType, Range)> + Default,
  {
    // clone action state to avoid modifying the original one
    let mut tmp_action_state = self.action_state.clone();

    let output = Self::lex_with_stateless(
      &self.stateless,
      &self.state,
      &mut tmp_action_state, // don't use self.action_state
      options.into(),
    );

    // don't update lexer state

    // TODO: prevent re-constructing the output?
    let output = LexOutput {
      digested: output.digested,
      token: output.token,
      re_lexable: Fork::ReLexableFactoryType::into_re_lexable(output.re_lexable, &self),
      err_acc: output.err_acc,
    };

    (output, tmp_action_state)
  }

  /// Try to yield the next token with the default options.
  /// [`Self::state`] and [`Self::action_state`] will be updated.
  pub fn lex(&mut self) -> LexOutput<Token<Kind>, ErrAcc, ()>
  where
    Kind: TokenKindIdProvider<Kind>,
    ErrAcc: Accumulator<(ErrorType, Range)> + Default,
  {
    self.lex_with_options(LexOptions::default())
  }

  /// Try to yield the next token with custom options.
  /// [`Self::state`] and [`Self::action_state`] will be updated.
  pub fn lex_with<'expect_literal, Fork: LexOptionsFork<'text, Kind, ActionState, ErrorType, ErrAcc>>(
    &mut self,
    options_builder: impl FnOnce(
      LexOptions<'expect_literal, Kind, ForkDisabled>,
    ) -> LexOptions<'expect_literal, Kind, Fork>,
  ) -> LexOutput<Token<Kind>, ErrAcc, <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, ActionState, ErrorType, ErrAcc>>::ReLexableType>
  where
    Kind: TokenKindIdProvider<Kind>,
    ErrAcc:Accumulator<(ErrorType, Range)>+Default
  {
    self.lex_with_options(options_builder(LexOptions::default()))
  }

  /// Try to yield the next token with custom options.
  /// [`Self::state`] and [`Self::action_state`] will be updated.
  pub fn lex_with_options<'expect_literal, Fork: LexOptionsFork<'text, Kind, ActionState, ErrorType, ErrAcc>>(
    &mut self,
    options: impl Into<LexOptions<'expect_literal, Kind, Fork>>,
  ) -> LexOutput<Token<Kind>, ErrAcc,<Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, ActionState, ErrorType, ErrAcc>>::ReLexableType>
  where
    Kind: TokenKindIdProvider<Kind>,
    ErrAcc:Accumulator<(ErrorType, Range)>+Default
  {
    let output = Self::lex_with_stateless(
      &self.stateless,
      &self.state,
      &mut self.action_state,
      options.into(),
    );

    // TODO: prevent re-constructing the output?
    let output = LexOutput {
      digested: output.digested,
      token: output.token,
      re_lexable: Fork::ReLexableFactoryType::into_re_lexable(output.re_lexable, &self),
      err_acc: output.err_acc,
    };

    // update state
    self.state.digest(output.digested);

    output
  }

  /// Digest the next `n` chars and set [`Self::action_state`].
  /// The caller should make sure `n` is smaller than the rest text length.
  pub fn digest_with(&mut self, n: usize, action_state: ActionState) -> &mut Self {
    self.state.digest(n);
    self.action_state = action_state;
    self
  }

  /// Digest the next `n` chars and set [`Self::action_state`] to default.
  /// The caller should make sure `n` is smaller than the rest text length.
  pub fn digest(&mut self, n: usize) -> &mut Self
  where
    ActionState: Default,
  {
    self.digest_with(n, ActionState::default())
  }

  /// Lex with muted actions.
  /// Returns [`None`] if the lexer is already trimmed.
  pub fn trim(&mut self) -> Option<TrimOutput<ErrAcc>>
  where
    Kind: TokenKindIdProvider<Kind>,
    ErrAcc: Accumulator<(ErrorType, Range)> + Default,
  {
    if self.state.trimmed() {
      return None;
    }

    let res = self.stateless.trim_with_options(
      self.state.text(),
      StatelessTrimOptions {
        start: self.state.digested(),
        action_state: &mut self.action_state,
        err_acc: ErrAcc::default(),
      },
    );

    self.state.trim(res.digested);

    Some(res)
  }

  fn lex_with_stateless<'expect_literal, Fork: LexOptionsFork<'text, Kind, ActionState, ErrorType, ErrAcc>>(
    stateless: &Rc<StatelessLexer<Kind, ActionState, ErrorType>>,
    state: &LexerState<'text>,
    action_state: &mut ActionState,
    options: LexOptions<'expect_literal, Kind, Fork>,
  ) -> LexOutput<Token<Kind>,ErrAcc, <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, ActionState, ErrorType, ErrAcc>>::StatelessReLexableType>
  where
    Kind: TokenKindIdProvider<Kind>,
    ErrAcc:Accumulator<(ErrorType, Range)>+Default,
  {
    stateless.lex_with_options(
      state.text(),
      StatelessLexOptions {
        start: state.digested(),
        action_state,
        err_acc: ErrAcc::default(),
        base: options,
      },
    )
  }
}
