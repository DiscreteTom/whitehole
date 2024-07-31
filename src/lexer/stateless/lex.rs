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
  /// Lex from the start of the input text with the default action state and options.
  ///
  /// This function will create a new action state and return it.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder};
  /// # let stateless = LexerBuilder::new().append(exact("1")).build_stateless();
  /// let (output, action_state) = stateless.lex("123");
  /// ```
  #[inline]
  pub fn lex<'text>(&self, text: &'text str) -> (LexOutput<Token<Kind>, (), ()>, ActionState)
  where
    Kind: TokenKindIdProvider<TokenKind = Kind>,
    ActionState: Default,
  {
    let mut action_state = ActionState::default();
    (
      self.lex_with(text, |o| o.action_state(&mut action_state)),
      action_state,
    )
  }

  /// Lex with the given options builder.
  /// # Panics
  /// Panics if no action is defined for the expected kind or literal.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder};
  /// # let stateless = LexerBuilder::new().append(exact("2")).build_stateless();
  /// # let mut action_state = ();
  /// stateless.lex_with("123", |o| o.action_state(&mut action_state));
  /// ```
  #[inline]
  pub fn lex_with<
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
      &'action_state mut ActionState,
      ErrAcc,
      Fork,
    >
  ) -> LexOutput<
    Token<Kind>,
    ErrAcc,
    <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, ActionState, ErrorType>>::StatelessReLexableType
  >
  where
    Kind: TokenKindIdProvider<TokenKind = Kind>,
    ActionState: 'action_state,
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
  /// # let mut action_state = ();
  /// let options = StatelessLexOptions::new().action_state(&mut action_state);
  /// stateless.lex_with_options("123", options);
  /// ```
  pub fn lex_with_options<
    'text,
    'expect_literal,
    'action_state,
    ErrAcc: Accumulator<(ErrorType, Range)>,
    Fork: LexOptionsFork<'text, Kind, ActionState, ErrorType>,
  >(
    &self,
    text: &'text str,
    options: StatelessLexOptions<'expect_literal, Kind, &'action_state mut ActionState, ErrAcc, Fork>,
  ) -> LexOutput<
    Token<Kind>,
    ErrAcc,
    <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, ActionState, ErrorType>>::StatelessReLexableType
  >
  where
    Kind: TokenKindIdProvider<TokenKind = Kind>,
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

      Self::execute_actions_mut(
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

      Self::execute_actions_mut(
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
