use super::{
  head_map::{HeadMap, HeadMapActions},
  literal_map::LiteralMap,
  options::StatelessLexOptions,
  utils::{break_loop_on_none, extract_token},
  StatelessLexer,
};
use crate::{
  lexer::{
    action::{ActionInput, ActionOutput},
    fork::LexOptionsFork,
    output::LexOutput,
    re_lex::{ReLexContext, ReLexableFactory},
    stateless::utils::{traverse_actions, traverse_actions_mut, update_state},
    token::{Range, Token, TokenKindId, TokenKindIdProvider},
  },
  utils::Accumulator,
};

impl<Kind, State, ErrorType> StatelessLexer<Kind, State, ErrorType> {
  const INVALID_EXPECTED_KIND: &'static str = "no action is defined for the expected kind";
  const INVALID_EXPECTED_LITERAL: &'static str = "no action is defined for the expected literal";

  /// Lex from the start of the input text with the default action state and options.
  ///
  /// This function will create a new action state and return it.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder};
  /// # let stateless = LexerBuilder::new().append(exact("1")).build_stateless();
  /// let (output, state) = stateless.lex("123");
  /// ```
  #[inline]
  pub fn lex<'text>(&self, text: &'text str) -> (LexOutput<Token<Kind>, (), ()>, State)
  where
    Kind: TokenKindIdProvider<TokenKind = Kind>,
    State: Default,
  {
    let mut state = State::default();
    (self.lex_with(text, |o| o.state(&mut state)), state)
  }

  /// Lex with the given options builder.
  /// # Panics
  /// Panics if no action is defined for the expected kind or literal.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder};
  /// # let stateless = LexerBuilder::new().append(exact("2")).build_stateless();
  /// # let mut state = ();
  /// stateless.lex_with("123", |o| o.state(&mut state));
  /// ```
  #[inline]
  pub fn lex_with<
    'text,
    'expect_literal,
    'state,
    ErrAcc: Accumulator<(ErrorType, Range)>,
    Fork: LexOptionsFork<'text, Kind, State, ErrorType>,
  >(
    &self,
    text: &'text str,
    options_builder: impl FnOnce(
      StatelessLexOptions<'expect_literal,Kind, (), (), ()>,
    ) -> StatelessLexOptions<
      'expect_literal,
      Kind,
      &'state mut State,
      ErrAcc,
      Fork,
    >
  ) -> LexOutput<
    Token<Kind>,
    ErrAcc,
    <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, State, ErrorType>>::StatelessReLexableType
  >
  where
    Kind: TokenKindIdProvider<TokenKind = Kind>,
    State: 'state,
  {
    self.lex_with_options(text, options_builder(StatelessLexOptions::new()))
  }

  /// Lex with the given [`StatelessLexOptions`].
  /// # Panics
  /// Panics if no action is defined for the expected kind or literal.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder, stateless::StatelessLexOptions};
  /// # let stateless = LexerBuilder::new().append(exact("2")).build_stateless();
  /// # let mut state = ();
  /// let options = StatelessLexOptions::new().state(&mut state);
  /// stateless.lex_with_options("123", options);
  /// ```
  pub fn lex_with_options<
    'text,
    'expect_literal,
    'state,
    ErrAcc: Accumulator<(ErrorType, Range)>,
    Fork: LexOptionsFork<'text, Kind, State, ErrorType>,
  >(
    &self,
    text: &'text str,
    options: StatelessLexOptions<'expect_literal, Kind, &'state mut State, ErrAcc, Fork>,
  ) -> LexOutput<
    Token<Kind>,
    ErrAcc,
    <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, State, ErrorType>>::StatelessReLexableType
  >
  where
    Kind: TokenKindIdProvider<TokenKind = Kind>,
  {
    if let Some(literal) = options.base.expectation.literal {
      let (literal_map, head_map) =
        self.get_literal_head_map(options.base.expectation.kind, literal);

      self.lex_mut_with_literal(
        literal_map,
        head_map,
        0,
        options.base.errors_to,
        options.start,
        text,
        options.state,
        literal,
        &options.base.re_lex,
        Fork::ReLexableFactoryType::default(),
        false,
      )
    } else {
      // else, no expected literal
      let head_map = self.get_kind_head_map(options.base.expectation.kind);

      self.lex_mut_without_literal(
        head_map,
        0,
        options.base.errors_to,
        options.start,
        text,
        options.state,
        &options.base.re_lex,
        Fork::ReLexableFactoryType::default(),
        false,
      )
    }
  }

  // there is no `StatelessLexer::peek()` because it is just the same with `StatelessLexer::lex()`

  /// Peek with the given options builder.
  /// This will clone [`StatelessLexOptions::state`] if it is mutated.
  /// # Panics
  /// Panics if no action is defined for the expected kind or literal.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder};
  /// # let stateless = LexerBuilder::new().append(exact("2")).build_stateless();
  /// # let state = ();
  /// let (output, mutated_state) = stateless.peek_with("123", |o| o.state(&state));
  /// ```
  #[inline]
  pub fn peek_with<
    'text,
    'expect_literal,
    'state,
    ErrAcc: Accumulator<(ErrorType, Range)>,
    Fork: LexOptionsFork<'text, Kind, State, ErrorType>,
  >(
    &self,
    text: &'text str,
    options_builder: impl FnOnce(
      StatelessLexOptions<'expect_literal,Kind, (), (), ()>,
    ) -> StatelessLexOptions<
      'expect_literal,
      Kind,
      &'state State,
      ErrAcc,
      Fork,
    >
  ) -> (
    LexOutput<
      Token<Kind>,
      ErrAcc,
      <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, State, ErrorType>>::StatelessReLexableType
    >,
    Option<State>
  )
  where
    Kind: TokenKindIdProvider<TokenKind = Kind>,
    State: Clone + 'state,
  {
    self.peek_with_options(text, options_builder(StatelessLexOptions::new()))
  }

  /// Peek with the given [`StatelessLexOptions`].
  /// This will clone [`StatelessLexOptions::state`] if it is mutated.
  /// # Panics
  /// Panics if no action is defined for the expected kind or literal.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder, stateless::StatelessLexOptions};
  /// # let stateless = LexerBuilder::new().append(exact("2")).build_stateless();
  /// # let state = ();
  /// let options = StatelessLexOptions::new().state(&state);
  /// let (output, mutated_state) = stateless.peek_with_options("123", options);
  /// ```
  pub fn peek_with_options<
    'text,
    'expect_literal,
    'state,
    ErrAcc: Accumulator<(ErrorType, Range)>,
    Fork: LexOptionsFork<'text, Kind, State, ErrorType>,
  >(
    &self,
    text: &'text str,
    options: StatelessLexOptions<'expect_literal, Kind, &'state State, ErrAcc, Fork>,
  ) -> (
    LexOutput<
      Token<Kind>,
      ErrAcc,
      <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, State, ErrorType>>::StatelessReLexableType
    >,
    Option<State>
  )
  where
    Kind: TokenKindIdProvider<TokenKind = Kind>,
    State: Clone
  {
    let mut digested = 0;
    let mut errors = options.base.errors_to;
    let re_lexable_factory = Fork::ReLexableFactoryType::default();
    let mut new_state = None;

    if let Some(literal) = options.base.expectation.literal {
      let (literal_map, head_map) =
        self.get_literal_head_map(options.base.expectation.kind, literal);

      loop {
        let input_start = options.start + digested;
        let input = break_loop_on_none!(ActionInput::new(text, input_start, options.state));
        let actions = get_actions_by_literal_map(&input, literal, literal_map, head_map);
        let (res, state) = traverse_actions(input, actions, &options.base.re_lex);
        new_state = state;
        let (output, action_index, muted) = break_loop_on_none!(res);

        if let Some(token) = process_output(output, muted, input_start, &mut digested, &mut errors)
        {
          return (
            done_with_token(
              digested,
              token,
              re_lexable_factory,
              input_start,
              actions.len(),
              action_index,
              errors,
            ),
            new_state,
          );
        }

        // if action state is mutated, continue with mutable lexing
        if let Some(mut state) = new_state {
          return (
            self.lex_mut_with_literal(
              literal_map,
              head_map,
              digested,
              errors,
              options.start,
              text,
              &mut state,
              literal,
              &options.base.re_lex,
              re_lexable_factory,
              true,
            ),
            Some(state),
          );
        }

        // else, muted, continue
      }
    } else {
      // else, no expected literal
      let head_map = self.get_kind_head_map(options.base.expectation.kind);

      loop {
        let input_start = options.start + digested;
        let input = break_loop_on_none!(ActionInput::new(text, input_start, options.state));
        let actions = head_map.get(input.next());
        let (res, state) = traverse_actions(input, actions, &options.base.re_lex);
        new_state = state;
        let (output, action_index, muted) = break_loop_on_none!(res);

        if let Some(token) = process_output(output, muted, input_start, &mut digested, &mut errors)
        {
          return (
            done_with_token(
              digested,
              token,
              re_lexable_factory,
              input_start,
              actions.len(),
              action_index,
              errors,
            ),
            new_state,
          );
        }

        // if action state is mutated, continue with mutable lexing
        if let Some(mut state) = new_state {
          return (
            self.lex_mut_without_literal(
              head_map,
              digested,
              errors,
              options.start,
              text,
              &mut state,
              &options.base.re_lex,
              re_lexable_factory,
              true,
            ),
            Some(state),
          );
        }

        // else, muted, continue
      }
    }

    // no more input or no accepted actions
    return (done_without_token(digested, errors), new_state);
  }

  fn get_literal_head_map(
    &self,
    kind: Option<&'static TokenKindId<Kind>>,
    literal: &str,
  ) -> (
    &LiteralMap<Kind, State, ErrorType>,
    &HeadMap<Kind, State, ErrorType>,
  ) {
    let literal_map = kind.map_or(&self.literal_map, |kind| {
      self
        .kind_literal_map
        .get(&kind)
        .expect(Self::INVALID_EXPECTED_KIND)
    });
    let head_map = literal_map
      .known_map()
      .get(literal)
      .expect(Self::INVALID_EXPECTED_LITERAL);
    (literal_map, head_map)
  }

  fn get_kind_head_map(
    &self,
    kind: Option<&'static TokenKindId<Kind>>,
  ) -> &HeadMap<Kind, State, ErrorType> {
    kind.map_or(
      &self.head_map, // if no expected kind, use the head map with all actions
      |kind| {
        self
          .kind_head_map
          .get(&kind)
          .expect(Self::INVALID_EXPECTED_KIND)
      },
    )
  }

  fn lex_mut_with_literal<
    'text,
    ErrAcc: Accumulator<(ErrorType, Range)>,
    ReLexableFactoryType: ReLexableFactory<'text, Kind, State, ErrorType>,
  >(
    &self,
    literal_map: &LiteralMap<Kind, State, ErrorType>,
    head_map: &HeadMap<Kind, State, ErrorType>,
    mut digested: usize,
    mut errors: ErrAcc,
    start: usize,
    text: &'text str,
    state: &mut State,
    literal: &str,
    re_lex: &ReLexContext,
    mut re_lexable_factory: ReLexableFactoryType,
    peek: bool,
  ) -> LexOutput<Token<Kind>, ErrAcc, ReLexableFactoryType::StatelessReLexableType>
  where
    Kind: TokenKindIdProvider<TokenKind = Kind>,
  {
    loop {
      let input_start = start + digested;
      let input = break_loop_on_none!(ActionInput::new(text, input_start, &mut *state));
      let actions = get_actions_by_literal_map(&input, literal, literal_map, head_map);
      let res = traverse_actions_mut(input, actions, re_lex, &mut re_lexable_factory, peek);
      let (output, action_index, muted) = break_loop_on_none!(res);

      if let Some(token) = process_output(output, muted, input_start, &mut digested, &mut errors) {
        return done_with_token(
          digested,
          token,
          re_lexable_factory,
          input_start,
          actions.len(),
          action_index,
          errors,
        );
      }

      // else, muted, continue
    }

    // no more input or no accepted actions
    return done_without_token(digested, errors);
  }

  fn lex_mut_without_literal<
    'text,
    ErrAcc: Accumulator<(ErrorType, Range)>,
    ReLexableFactoryType: ReLexableFactory<'text, Kind, State, ErrorType>,
  >(
    &self,
    head_map: &HeadMap<Kind, State, ErrorType>,
    mut digested: usize,
    mut errors: ErrAcc,
    start: usize,
    text: &'text str,
    state: &mut State,
    re_lex: &ReLexContext,
    mut re_lexable_factory: ReLexableFactoryType,
    peek: bool,
  ) -> LexOutput<Token<Kind>, ErrAcc, ReLexableFactoryType::StatelessReLexableType>
  where
    Kind: TokenKindIdProvider<TokenKind = Kind>,
  {
    loop {
      let input_start = start + digested;
      let input = break_loop_on_none!(ActionInput::new(text, input_start, &mut *state));
      let actions = head_map.get(input.next());
      let res = traverse_actions_mut(input, actions, re_lex, &mut re_lexable_factory, peek);
      let (output, action_index, muted) = break_loop_on_none!(res);

      if let Some(token) = process_output(output, muted, input_start, &mut digested, &mut errors) {
        return done_with_token(
          digested,
          token,
          re_lexable_factory,
          input_start,
          actions.len(),
          action_index,
          errors,
        );
      }

      // else, muted, continue
    }

    // no more input or no accepted actions
    return done_without_token(digested, errors);
  }
}

fn done_with_token<
  'text,
  Kind: 'static,
  State,
  ErrorType,
  ErrAcc,
  ReLexableFactoryType: ReLexableFactory<'text, Kind, State, ErrorType>,
>(
  digested: usize,
  token: Token<Kind>,
  re_lexable_factory: ReLexableFactoryType,
  input_start: usize,
  actions_len: usize,
  action_index: usize,
  errors: ErrAcc,
) -> LexOutput<Token<Kind>, ErrAcc, ReLexableFactoryType::StatelessReLexableType> {
  LexOutput {
    digested,
    token: Some(token),
    re_lexable: re_lexable_factory.into_stateless_re_lexable(
      input_start,
      actions_len,
      action_index,
    ),
    errors,
  }
}

fn get_actions_by_literal_map<'this, Kind: 'static, State, StateRef, ErrorType>(
  input: &ActionInput<StateRef>,
  literal: &str,
  literal_map: &'this LiteralMap<Kind, State, ErrorType>,
  head_map: &'this HeadMap<Kind, State, ErrorType>,
) -> &'this HeadMapActions<Kind, State, ErrorType> {
  {
    if !input.rest().starts_with(literal) {
      // prefix mismatch, only execute muted actions
      literal_map.muted_map()
    } else {
      // prefix match, use the literal's head map
      head_map
    }
  }
  .get(input.next())
}

/// Process the output, update the digested, collect errors, and emit token if not muted.
/// Return the token if not muted, otherwise return [`None`].
fn process_output<Kind, ErrorType, ErrAcc: Accumulator<(ErrorType, Range)>>(
  output: ActionOutput<Kind, Option<ErrorType>>,
  muted: bool,
  start: usize,
  digested: &mut usize,
  errors: &mut ErrAcc,
) -> Option<Token<Kind>> {
  update_state(output.digested, output.error, start, digested, errors);
  extract_token(output.kind, output.digested, muted, start)
}

#[inline]
fn done_without_token<TokenType, ErrAcc, ReLexableType: Default>(
  digested: usize,
  errors: ErrAcc,
) -> LexOutput<TokenType, ErrAcc, ReLexableType> {
  LexOutput {
    digested,
    token: None,
    re_lexable: ReLexableType::default(),
    errors,
  }
}

// TODO: add tests
// #[cfg(test)]
// mod tests {
//   use crate::lexer::{action::exact, LexerBuilder};
//   use whitehole_macros::_TokenKind;
//   use MyKind::*;

//   #[derive(_TokenKind, Clone)]
//   enum MyKind {
//     A,
//     B,
//   }

//   #[test]
//   #[should_panic]
//   fn stateless_lexer_lex_with_unknown_kind() {
//     let stateless = LexerBuilder::<MyKind>::new()
//       .define(A, exact("A"))
//       .build_stateless();
//     stateless.lex_with("A", &mut (), |o| o.expect(B));
//   }
// }
