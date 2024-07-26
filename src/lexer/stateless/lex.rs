use super::{
  options::StatelessLexOptions, output::StatelessOutput, StatelessLexer, StatelessTrimOptions,
};
use crate::{
  lexer::{
    fork::{ForkDisabled, LexOptionsFork},
    output::{LexOutput, TrimOutput},
    re_lex::{ReLexContext, ReLexableFactory},
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
    Kind: TokenKindIdProvider<Kind>,
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
      StatelessLexOptions<'expect_literal,Kind, (), (), ForkDisabled>,
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
    Kind: TokenKindIdProvider<Kind>,
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
    Kind: TokenKindIdProvider<Kind>,
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
        |input| {
          let literal_mismatch = !input.rest().starts_with(literal);
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
        LexOutput::new(options.base.errors_to),
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
        LexOutput::new(options.base.errors_to),
      )
    }
  }

  /// Lex with muted actions, the default action state and the default options.
  ///
  /// This function will create a new action state and return it.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder, stateless::StatelessLexOptions};
  /// # let stateless = LexerBuilder::new().append(exact("2")).build_stateless();
  /// let (output, action_state) = stateless.trim("123");
  /// ```
  #[inline]
  pub fn trim<'text>(&self, text: &'text str) -> (TrimOutput<()>, ActionState)
  where
    Kind: TokenKindIdProvider<Kind>,
    ActionState: Default,
  {
    let mut action_state = ActionState::default();
    (
      self.trim_with(text, |o| o.action_state(&mut action_state)),
      action_state,
    )
  }

  /// Lex with muted actions and the given options builder.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder};
  /// # let stateless = LexerBuilder::new().append(exact("2")).build_stateless();
  /// # let mut action_state = ();
  /// stateless.trim_with("123", |o| o.action_state(&mut action_state));
  /// ```
  #[inline]
  pub fn trim_with<'text, 'action_state, ErrAcc: Accumulator<(ErrorType, Range)>>(
    &self,
    text: &'text str,
    options_builder: impl FnOnce(
      StatelessTrimOptions<(), ()>,
    ) -> StatelessTrimOptions<&'action_state mut ActionState, ErrAcc>,
  ) -> TrimOutput<ErrAcc>
  where
    Kind: TokenKindIdProvider<Kind>,
    ActionState: 'action_state,
  {
    self.trim_with_options(text, options_builder(StatelessTrimOptions::new()))
  }

  /// Lex with muted actions and the given [`StatelessLexOptions`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder, stateless::StatelessTrimOptions};
  /// # let stateless = LexerBuilder::new().append(exact("2")).build_stateless();
  /// # let mut action_state = ();
  /// let options = StatelessTrimOptions::new().action_state(&mut action_state);
  /// stateless.trim_with_options("123", options);
  /// ```
  pub fn trim_with_options<'text, 'action_state, ErrAcc: Accumulator<(ErrorType, Range)>>(
    &self,
    text: &'text str,
    options: StatelessTrimOptions<&'action_state mut ActionState, ErrAcc>,
  ) -> TrimOutput<ErrAcc>
  where
    Kind: TokenKindIdProvider<Kind>,
  {
    Self::execute_actions(
      // the literal map's muted map contains all the muted actions
      |_| self.literal_map.muted_map(),
      // there is no expectation options for trim,
      // with the same `start` and `action_state` all trims will get the same result,
      // so re-lex is meaningless, always use the default re-lex context
      &ReLexContext::default(),
      text,
      options.start,
      options.action_state,
      (),
      <TrimOutput<_> as StatelessOutput<(), _, ()>>::new(options.base.errors_to),
    )
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
