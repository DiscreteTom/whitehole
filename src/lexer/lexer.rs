use super::{
  fork::LexOptionsFork,
  instant::Instant,
  options::{LexOptions, TrimOptions},
  output::{LexOutput, TrimOutput},
  re_lex::ReLexableFactory,
  stateless::{StatelessLexOptions, StatelessLexer, StatelessTrimOptions},
  token::{Range, Token},
};
use crate::utils::Accumulator;
use std::rc::Rc;

/// This is the "stateful" lexer, it manages 2 states: the [`LexerState`] and the `State`.
/// The [`LexerState`] is responsible to manage the text and the position of the lexer.
/// The `State` is provided by you can be accessed by immutable [`Action`](crate::lexer::action::Action)s
/// to realize stateful lexing.
///
/// If you want a stateless experience, you can use [`StatelessLexer`].
///
/// To create a lexer, you should use [`LexerBuilder`](crate::lexer::LexerBuilder).
/// # Design
/// ## Why there is no `Lexer::errors` to store all the errors?
/// Why the error accumulator is not a field of [`Lexer`]
/// just like [`Lexer::state`],
/// but a field of [`LexOptions`] which needs to be provided every time?
///
/// [`Lexer::state`] is just a value, but the error accumulator is a collection/container.
/// We don't want unnecessary memory allocation, so we won't create the container
/// for users. Users can create their own accumulator and manage its memory allocation.
/// E.g. some users may just want to print the errors, so they don't need any container;
/// some users may want to process errors after each lexing, and clear the container
/// before next lexing to save memory; some users may want to store all errors
/// in a container and process them later.
pub struct Lexer<'text, Kind: 'static, State, ErrorType> {
  /// You can mutate this directly if needed.
  pub state: State,

  // use Rc so that this is clone-able
  stateless: Rc<StatelessLexer<Kind, State, ErrorType>>,
  instant: Instant<'text>,
}

impl<'text, Kind, State: Clone, ErrorType> Clone for Lexer<'text, Kind, State, ErrorType> {
  #[inline]
  fn clone(&self) -> Self {
    Self {
      state: self.state.clone(),
      stateless: self.stateless.clone(),
      instant: self.instant.clone(),
    }
  }
}

impl<'text, Kind, State, ErrorType> Lexer<'text, Kind, State, ErrorType> {
  /// Create a new lexer with the given stateless lexer, action state and text.
  /// For most cases you should use [`LexerBuilder`](crate::lexer::LexerBuilder)
  /// to create a lexer.
  #[inline]
  pub const fn new(
    stateless: Rc<StatelessLexer<Kind, State, ErrorType>>,
    state: State,
    text: &'text str,
  ) -> Self {
    Self {
      state,
      stateless,
      instant: Instant::new(text),
    }
  }

  /// Get the stateless lexer.
  #[inline]
  pub const fn stateless(&self) -> &Rc<StatelessLexer<Kind, State, ErrorType>> {
    &self.stateless
  }
  /// Get the lexer state.
  /// You are not allowed to mutate the lexer state directly.
  #[inline]
  pub const fn state(&self) -> &Instant<'text> {
    &self.instant
  }

  /// Clone self with a new action state.
  #[inline]
  pub fn clone_with(&self, state: State) -> Self {
    Self::new(self.stateless.clone(), state, self.instant.text())
  }

  /// Consume self, return a new lexer with the same actions and a new text.
  /// [`Self::state`] and [`Self::state`] will be reset to default.
  #[inline]
  pub fn reload<'new_text>(self, text: &'new_text str) -> Lexer<'new_text, Kind, State, ErrorType>
  where
    State: Default,
  {
    Lexer {
      stateless: self.stateless,
      instant: Instant::new(text),
      state: State::default(),
    }
  }

  /// Consume self, return a new lexer with the same actions, a new text and the given action state.
  /// [`Self::state`] will be reset to default.
  #[inline]
  pub fn reload_with<'new_text>(
    self,
    text: &'new_text str,
    state: State,
  ) -> Lexer<'new_text, Kind, State, ErrorType> {
    Lexer::new(self.stateless, state, text)
  }

  /// Peek the next token with the default options, without updating
  /// [`Self::state`] and [`Self::state`].
  ///
  /// If `State` is mutated in the lexing process,
  /// [`Self::state`] will be cloned and returned.
  #[inline]
  pub fn peek(&self) -> (LexOutput<Token<Kind>, (), ()>, Option<State>)
  where
    State: Clone,
  {
    self.peek_with_options(LexOptions::new())
  }

  /// Peek the next token with custom options, without updating
  /// [`Self::state`] and [`Self::state`].
  ///
  /// If `State` is mutated in the lexing process,
  #[inline]
  pub fn peek_with<'expect_literal, ErrAcc, Fork: LexOptionsFork<'text, Kind, State, ErrorType>>(
    &self,
    options_builder: impl FnOnce(
      LexOptions<'expect_literal, Kind, (), ()>,
    ) -> LexOptions<'expect_literal, Kind, ErrAcc, Fork>,
  ) -> (LexOutput<Token<Kind>, ErrAcc, <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, State, ErrorType>>::ReLexableType>, Option<State>)
  where
    State: Clone,
    ErrAcc: Accumulator<(ErrorType, Range)>,
  {
    self.peek_with_options(options_builder(LexOptions::new()))
  }

  /// Peek the next token with custom options, without updating
  /// [`Self::state`] and [`Self::state`].
  ///
  /// [`Self::state`] will be cloned and returned.
  pub fn peek_with_options<'expect_literal, ErrAcc, Fork: LexOptionsFork<'text, Kind, State, ErrorType>>(
    &self,
    options: impl Into<LexOptions<'expect_literal, Kind, ErrAcc, Fork>>,
  ) -> (LexOutput<Token<Kind>, ErrAcc, <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, State, ErrorType>>::ReLexableType>, Option<State>)
  where
    State: Clone,
    ErrAcc: Accumulator<(ErrorType, Range)>,
  {
    let (output, new_state) = self.stateless.peek_with_options(
      self.instant.text(),
      StatelessLexOptions {
        start: self.instant.digested(),
        state: &self.state,
        base: options.into(),
      },
    );

    // TODO: prevent re-constructing the output?
    let output = LexOutput {
      digested: output.digested,
      token: output.token,
      re_lexable: Fork::ReLexableFactoryType::build_re_lexable(
        output.re_lexable,
        output.digested,
        self,
      ),
      errors: output.errors,
    };

    (output, new_state)

    // don't update lexer state
  }

  /// Try to yield the next token with the default options.
  /// [`Self::state`] and [`Self::state`] will be updated.
  #[inline]
  pub fn lex(&mut self) -> LexOutput<Token<Kind>, (), ()> {
    self.lex_with_options(LexOptions::new())
  }

  /// Try to yield the next token with custom options.
  /// [`Self::state`] and [`Self::state`] will be updated.
  #[inline]
  pub fn lex_with<'expect_literal, ErrAcc, Fork: LexOptionsFork<'text, Kind, State, ErrorType>>(
    &mut self,
    options_builder: impl FnOnce(
      LexOptions<'expect_literal, Kind, (), ()>,
    ) -> LexOptions<'expect_literal, Kind, ErrAcc, Fork>,
  ) -> LexOutput<
    Token<Kind>,
    ErrAcc,
    <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, State, ErrorType>>::ReLexableType,
  >
  where
    ErrAcc: Accumulator<(ErrorType, Range)> + Default,
  {
    self.lex_with_options(options_builder(LexOptions::new()))
  }

  /// Try to yield the next token with custom options.
  /// [`Self::state`] and [`Self::state`] will be updated.
  pub fn lex_with_options<
    'expect_literal,
    ErrAcc,
    Fork: LexOptionsFork<'text, Kind, State, ErrorType>,
  >(
    &mut self,
    options: impl Into<LexOptions<'expect_literal, Kind, ErrAcc, Fork>>,
  ) -> LexOutput<
    Token<Kind>,
    ErrAcc,
    <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, State, ErrorType>>::ReLexableType,
  >
  where
    ErrAcc: Accumulator<(ErrorType, Range)> + Default,
  {
    let output = Self::lex_with_stateless(
      &self.stateless,
      &self.instant,
      &mut self.state,
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
    self.instant.digest(output.digested);

    output
  }

  /// Digest the next `n` chars and optionally set [`Self::state`].
  /// The caller should make sure `n` is smaller than the rest text length.
  #[inline]
  pub fn digest_with(&mut self, n: usize, state: impl Into<Option<State>>) -> &mut Self {
    self.instant.digest(n);
    if let Some(state) = state.into() {
      self.state = state;
    }
    self
  }

  /// Digest the next `n` chars and set [`Self::state`] to default.
  /// The caller should make sure `n` is smaller than the rest text length.
  #[inline]
  pub fn digest(&mut self, n: usize) -> &mut Self
  where
    State: Default,
  {
    self.digest_with(n, State::default())
  }

  /// Lex with muted actions.
  /// Returns [`None`] if the lexer is already trimmed.
  pub fn trim<ErrAcc>(&mut self, err_acc: ErrAcc) -> Option<TrimOutput<ErrAcc>>
  where
    ErrAcc: Accumulator<(ErrorType, Range)>,
  {
    if self.instant.trimmed() {
      return None;
    }

    let res = self.stateless.trim_with_options(
      self.instant.text(),
      StatelessTrimOptions {
        start: self.instant.digested(),
        state: &mut self.state,
        base: TrimOptions::new().errors_to(err_acc),
      },
    );

    self.instant.trim(res.digested);

    Some(res)
  }
  // TODO: add trim_with/trim_with_options

  #[inline]
  fn lex_with_stateless<'expect_literal, ErrAcc,Fork: LexOptionsFork<'text, Kind, State, ErrorType>>(
    stateless: &Rc<StatelessLexer<Kind, State, ErrorType>>,
    instant: &Instant<'text>,
    state: &mut State,
    options: LexOptions<'expect_literal, Kind, ErrAcc,Fork>,
  ) -> LexOutput<Token<Kind>,ErrAcc, <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, State, ErrorType>>::StatelessReLexableType>
  where
    ErrAcc:Accumulator<(ErrorType, Range)>,
  {
    stateless.lex_with_options(
      instant.text(),
      StatelessLexOptions {
        start: instant.digested(),
        state,
        base: options,
      },
    )
  }

  pub(crate) fn from_re_lexable(
    stateless: Rc<StatelessLexer<Kind, State, ErrorType>>,
    state: State,
    instant: Instant<'text>,
  ) -> Self {
    Lexer {
      stateless,
      instant,
      state,
    }
  }
}
