use super::{options::StatelessLexOptions, output::StatelessOutputFactory, StatelessLexer};
use crate::{
  lexer::{
    fork::LexOptionsFork,
    output::LexOutput,
    re_lex::ReLexableFactory,
    stateless::output::LexOutputBuilder,
    token::{Range, Token, TokenKindIdProvider},
  },
  utils::Accumulator,
};

impl<Kind, ActionState, ErrorType> StatelessLexer<Kind, ActionState, ErrorType> {
  // there is no `StatelessLexer::peek()` because it is just the same with `StatelessLexer::lex()`

  /// Peek with the given options builder.
  /// This will clone [`StatelessLexOptions::action_state`] if it is mutated.
  /// # Panics
  /// Panics if no action is defined for the expected kind or literal.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder};
  /// # let stateless = LexerBuilder::new().append(exact("2")).build_stateless();
  /// # let action_state = ();
  /// let (output, mutated_action_state) = stateless.peek_with("123", |o| o.action_state(&action_state));
  /// ```
  #[inline]
  pub fn peek_with<
    'text,
    'expect_literal,
    'action_state,
    ErrAcc: Accumulator<(ErrorType, Range)>,
    Fork: LexOptionsFork<'text, Kind, ActionState, ErrorType>,
  >(
    &self,
    text: &'text str,
    options_builder: impl FnOnce(
      StatelessLexOptions<'expect_literal,Kind, (), (), ()>,
    ) -> StatelessLexOptions<
      'expect_literal,
      Kind,
      &'action_state ActionState,
      ErrAcc,
      Fork,
    >
  ) -> (
    LexOutput<
      Token<Kind>,
      ErrAcc,
      <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, ActionState, ErrorType>>::StatelessReLexableType
    >,
    Option<ActionState>
  )
  where
    Kind: TokenKindIdProvider<TokenKind = Kind>,
    ActionState: Clone + 'action_state,
  {
    self.peek_with_options(text, options_builder(StatelessLexOptions::new()))
  }

  /// Peek with the given [`StatelessLexOptions`].
  /// This will clone [`StatelessLexOptions::action_state`] if it is mutated.
  /// # Panics
  /// Panics if no action is defined for the expected kind or literal.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder, stateless::StatelessLexOptions};
  /// # let stateless = LexerBuilder::new().append(exact("2")).build_stateless();
  /// # let action_state = ();
  /// let options = StatelessLexOptions::new().action_state(&action_state);
  /// let (output, mutated_action_state) = stateless.peek_with_options("123", options);
  /// ```
  pub fn peek_with_options<
    'text,
    'expect_literal,
    'action_state,
    ErrAcc: Accumulator<(ErrorType, Range)>,
    Fork: LexOptionsFork<'text, Kind, ActionState, ErrorType>,
  >(
    &self,
    text: &'text str,
    options: StatelessLexOptions<'expect_literal, Kind, &'action_state ActionState, ErrAcc, Fork>,
  ) -> (
    LexOutput<
      Token<Kind>,
      ErrAcc,
      <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, ActionState, ErrorType>>::StatelessReLexableType
    >,
    Option<ActionState>
  )
  where
    Kind: TokenKindIdProvider<TokenKind = Kind>,
    ActionState: Clone
  {
    const INVALID_EXPECTED_KIND: &str = "no action is defined for the expected kind";

    if let Some(literal) = options.base.expectation.literal {
      let literal_map = options
        .base
        .expectation
        .kind
        .map_or(&self.literal_map, |kind| {
          self
            .kind_literal_map
            .get(&kind)
            .expect(INVALID_EXPECTED_KIND)
        });
      let head_map = literal_map
        .known_map()
        .get(literal)
        .expect("no action is defined for the expected literal");

      Self::execute_actions(
        |rest| {
          let literal_mismatch = !rest.starts_with(literal);
          if literal_mismatch {
            literal_map.muted_map()
          } else {
            head_map
          }
        },
        &options.base.re_lex,
        text,
        options.start,
        options.action_state,
        Fork::ReLexableFactoryType::default(),
        <
          LexOutputBuilder<ErrAcc> as StatelessOutputFactory<
            Token<Kind>,
            _,
            <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, ActionState, ErrorType>>::StatelessReLexableType
          >
        >::new(options.base.errors_to),
      )
    } else {
      // else, no expected literal
      let head_map = options.base.expectation.kind.map_or(
        &self.head_map, // if no expected kind, use the head map with all actions
        |kind| self.kind_head_map.get(&kind).expect(INVALID_EXPECTED_KIND),
      );

      Self::execute_actions(
        |_| head_map,
        &options.base.re_lex,
        text,
        options.start,
        options.action_state,
        Fork::ReLexableFactoryType::default(),
        <
          LexOutputBuilder<ErrAcc> as StatelessOutputFactory<
            Token<Kind>,
            _,
            <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, ActionState, ErrorType>>::StatelessReLexableType
          >
        >::new(options.base.errors_to),
      )
    }
  }
}
