use super::{
  fork::LexOptionsFork,
  options::{LexOptions, TrimOptions},
  output::{LexOutput, TrimOutput},
  re_lex::ReLexableFactory,
  state::LexerState,
  stateless::{StatelessLexOptions, StatelessLexer, StatelessTrimOptions},
  token::{Range, Token, TokenKindIdProvider},
};
use crate::utils::Accumulator;
use std::rc::Rc;

/// This is the "stateful" lexer, it manages 2 states: the [`LexerState`] and the `ActionState`.
/// The [`LexerState`] is responsible to manage the text and the position of the lexer.
/// The `ActionState` is provided by you can be accessed by immutable [`Action`](crate::lexer::action::Action)s
/// to realize stateful lexing.
///
/// If you want a stateless experience, you can use [`StatelessLexer`].
///
/// To create a lexer, you should use [`LexerBuilder`](crate::lexer::LexerBuilder).
/// # Design
/// ## Why there is no `Lexer::errors` to store all the errors?
/// Why the error accumulator is not a field of [`Lexer`]
/// just like [`Lexer::action_state`],
/// but a field of [`LexOptions`] which needs to be provided every time?
///
/// [`Lexer::action_state`] is just a value, but the error accumulator is a collection/container.
/// We don't want unnecessary memory allocation, so we won't create the container
/// for users. Users can create their own accumulator and manage its memory allocation.
/// E.g. some users may just want to print the errors, so they don't need any container;
/// some users may want to process errors after each lexing, and clear the container
/// before next lexing to save memory; some users may want to store all errors
/// in a container and process them later.
pub struct Lexer<'text, Kind: 'static, ActionState, ErrorType> {
  /// You can mutate this directly if needed.
  pub action_state: ActionState,

  // use Rc so that this is clone-able
  stateless: Rc<StatelessLexer<Kind, ActionState, ErrorType>>,
  state: LexerState<'text>,
}

impl<'text, Kind, ActionState: Clone, ErrorType> Clone
  for Lexer<'text, Kind, ActionState, ErrorType>
{
  #[inline]
  fn clone(&self) -> Self {
    Self {
      action_state: self.action_state.clone(),
      stateless: self.stateless.clone(),
      state: self.state.clone(),
    }
  }
}

impl<'text, Kind, ActionState, ErrorType> Lexer<'text, Kind, ActionState, ErrorType> {
  /// Create a new lexer with the given stateless lexer, action state and text.
  /// For most cases you should use [`LexerBuilder`](crate::lexer::LexerBuilder)
  /// to create a lexer.
  #[inline]
  pub const fn new(
    stateless: Rc<StatelessLexer<Kind, ActionState, ErrorType>>,
    action_state: ActionState,
    text: &'text str,
  ) -> Self {
    Self {
      action_state,
      stateless,
      state: LexerState::new(text),
    }
  }

  /// Get the stateless lexer.
  #[inline]
  pub const fn stateless(&self) -> &Rc<StatelessLexer<Kind, ActionState, ErrorType>> {
    &self.stateless
  }
  /// Get the lexer state.
  /// You are not allowed to mutate the lexer state directly.
  #[inline]
  pub const fn state(&self) -> &LexerState<'text> {
    &self.state
  }

  /// Clone self with a new action state.
  #[inline]
  pub fn clone_with(&self, action_state: ActionState) -> Self {
    Self::new(self.stateless.clone(), action_state, self.state.text())
  }

  /// Consume self, return a new lexer with the same actions and a new text.
  /// [`Self::state`] and [`Self::action_state`] will be reset to default.
  #[inline]
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
  #[inline]
  pub fn reload_with<'new_text>(
    self,
    text: &'new_text str,
    action_state: ActionState,
  ) -> Lexer<'new_text, Kind, ActionState, ErrorType> {
    Lexer::new(self.stateless, action_state, text)
  }

  /// Peek the next token with the default options, without updating
  /// [`Self::state`] and [`Self::action_state`].
  /// This will clone [`Self::action_state`] and return it.
  #[inline]
  pub fn peek(&self) -> (LexOutput<Token<Kind>, (), ()>, ActionState)
  where
    Kind: TokenKindIdProvider<TokenKind = Kind>,
    ActionState: Clone,
  {
    self.peek_with_options(LexOptions::new())
  }

  /// Peek the next token with custom options, without updating
  /// [`Self::state`] and [`Self::action_state`].
  /// This will clone the [`Self::action_state`] and return it.
  #[inline]
  pub fn peek_with<
    'expect_literal,
    ErrAcc,
    Fork: LexOptionsFork<'text, Kind, ActionState, ErrorType>,
  >(
    &self,
    options_builder: impl FnOnce(
      LexOptions<'expect_literal, Kind, (), ()>,
    ) -> LexOptions<'expect_literal, Kind, ErrAcc, Fork>,
  ) -> (
    LexOutput<
      Token<Kind>,
      ErrAcc,
      <Fork::ReLexableFactoryType as ReLexableFactory<
        'text,
        Kind,
        ActionState,
        ErrorType,
      >>::ReLexableType,
    >,
    ActionState,
  )
  where
    Kind: TokenKindIdProvider<TokenKind = Kind>,
    ActionState: Clone,
    ErrAcc: Accumulator<(ErrorType, Range)>,
  {
    self.peek_with_options(options_builder(LexOptions::new()))
  }

  /// Peek the next token with custom options, without updating
  /// [`Self::state`] and [`Self::action_state`].
  /// This will clone the [`Self::action_state`] and return it.
  pub fn peek_with_options<
    'expect_literal,
    ErrAcc,
    Fork: LexOptionsFork<'text, Kind, ActionState, ErrorType>,
  >(
    &self,
    options: impl Into<LexOptions<'expect_literal, Kind, ErrAcc, Fork>>,
  ) -> (
    LexOutput<
      Token<Kind>,
      ErrAcc,
      <Fork::ReLexableFactoryType as ReLexableFactory<
        'text,
        Kind,
        ActionState,
        ErrorType,
      >>::ReLexableType,
    >,
    ActionState,
  )
  where
    Kind: TokenKindIdProvider<TokenKind = Kind>,
    ActionState: Clone,
    ErrAcc: Accumulator<(ErrorType, Range)>,
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
      re_lexable: Fork::ReLexableFactoryType::build_re_lexable(
        output.re_lexable,
        output.digested,
        &self,
      ),
      errors: output.errors,
    };

    (output, tmp_action_state)
  }

  /// Try to yield the next token with the default options.
  /// [`Self::state`] and [`Self::action_state`] will be updated.
  #[inline]
  pub fn lex(&mut self) -> LexOutput<Token<Kind>, (), ()>
  where
    Kind: TokenKindIdProvider<TokenKind = Kind>,
  {
    self.lex_with_options(LexOptions::new())
  }

  /// Try to yield the next token with custom options.
  /// [`Self::state`] and [`Self::action_state`] will be updated.
  #[inline]
  pub fn lex_with<'expect_literal, ErrAcc, Fork: LexOptionsFork<'text, Kind, ActionState, ErrorType>>(
    &mut self,
    options_builder: impl FnOnce(
      LexOptions<'expect_literal, Kind, (),()>,
    ) -> LexOptions<'expect_literal, Kind, ErrAcc,Fork>,
  ) -> LexOutput<Token<Kind>, ErrAcc, <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, ActionState, ErrorType>>::ReLexableType>
  where
    Kind: TokenKindIdProvider<TokenKind = Kind>,
    ErrAcc:Accumulator<(ErrorType, Range)>+Default
  {
    self.lex_with_options(options_builder(LexOptions::new()))
  }

  /// Try to yield the next token with custom options.
  /// [`Self::state`] and [`Self::action_state`] will be updated.
  pub fn lex_with_options<'expect_literal, ErrAcc,Fork: LexOptionsFork<'text, Kind, ActionState, ErrorType>>(
    &mut self,
    options: impl Into<LexOptions<'expect_literal, Kind,ErrAcc, Fork>>,
  ) -> LexOutput<Token<Kind>, ErrAcc,<Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, ActionState, ErrorType>>::ReLexableType>
  where
    Kind: TokenKindIdProvider<TokenKind = Kind>,
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
      re_lexable: Fork::ReLexableFactoryType::build_re_lexable(
        output.re_lexable,
        output.digested,
        &self,
      ),
      errors: output.errors,
    };

    // update state
    self.state.digest(output.digested);

    output
  }

  /// Digest the next `n` chars and set [`Self::action_state`].
  /// The caller should make sure `n` is smaller than the rest text length.
  #[inline]
  pub fn digest_with(&mut self, n: usize, action_state: ActionState) -> &mut Self {
    self.state.digest(n);
    self.action_state = action_state;
    self
  }

  /// Digest the next `n` chars and set [`Self::action_state`] to default.
  /// The caller should make sure `n` is smaller than the rest text length.
  #[inline]
  pub fn digest(&mut self, n: usize) -> &mut Self
  where
    ActionState: Default,
  {
    self.digest_with(n, ActionState::default())
  }

  /// Lex with muted actions.
  /// Returns [`None`] if the lexer is already trimmed.
  pub fn trim<ErrAcc>(&mut self, err_acc: ErrAcc) -> Option<TrimOutput<ErrAcc>>
  where
    Kind: TokenKindIdProvider<TokenKind = Kind>,
    ErrAcc: Accumulator<(ErrorType, Range)>,
  {
    if self.state.trimmed() {
      return None;
    }

    let res = self.stateless.trim_with_options(
      self.state.text(),
      StatelessTrimOptions {
        start: self.state.digested(),
        action_state: &mut self.action_state,
        base: TrimOptions::new().errors_to(err_acc),
      },
    );

    self.state.trim(res.digested);

    Some(res)
  }
  // TODO: add trim_with/trim_with_options

  #[inline]
  fn lex_with_stateless<'expect_literal, ErrAcc,Fork: LexOptionsFork<'text, Kind, ActionState, ErrorType>>(
    stateless: &Rc<StatelessLexer<Kind, ActionState, ErrorType>>,
    state: &LexerState<'text>,
    action_state: &mut ActionState,
    options: LexOptions<'expect_literal, Kind, ErrAcc,Fork>,
  ) -> LexOutput<Token<Kind>,ErrAcc, <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, ActionState, ErrorType>>::StatelessReLexableType>
  where
    Kind: TokenKindIdProvider<TokenKind = Kind>,
    ErrAcc:Accumulator<(ErrorType, Range)>,
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

  pub(crate) fn from_re_lexable(
    stateless: Rc<StatelessLexer<Kind, ActionState, ErrorType>>,
    action_state: ActionState,
    state: LexerState<'text>,
  ) -> Self {
    Lexer {
      stateless,
      state,
      action_state,
    }
  }
}
