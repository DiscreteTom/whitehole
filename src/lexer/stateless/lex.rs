use super::{options::StatelessLexOptions, StatelessLexer};
use crate::{
  lexer::{
    action::{ActionInput, ActionOutput},
    fork::LexOptionsFork,
    output::LexOutput,
    re_lex::ReLexableFactory,
    stateless::exec::{create_range, create_token, traverse_actions_mut},
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

    let mut digested = 0;
    let mut errors = options.base.errors_to;
    let mut re_lexable_factory = Fork::ReLexableFactoryType::default();

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

      while let Some(((input_start, actions_len), (output, action_index, muted))) =
        ActionInput::new(text, options.start + digested, &mut *options.action_state).and_then(
          |mut input| {
            let actions = {
              if !input.rest().starts_with(literal) {
                // prefix mismatch, only execute muted actions
                literal_map.muted_map()
              } else {
                // prefix match, use the literal's head map
                head_map
              }
            }
            .get(input.next());

            traverse_actions_mut(
              &mut input,
              actions,
              &options.base.re_lex,
              &mut re_lexable_factory,
            )
            .map(|res| ((input.start(), actions.len()), res))
          },
        )
      {
        if let Some(token) = process_output(output, muted, input_start, &mut digested, &mut errors)
        {
          return LexOutput {
            digested,
            token: Some(token),
            re_lexable: re_lexable_factory.into_stateless_re_lexable(
              input_start,
              actions_len,
              action_index,
            ),
            errors,
          };
        }

        // else, muted, continue
      }
    } else {
      // else, no expected literal
      let head_map = options.base.expectation.kind.map_or(
        &self.head_map, // if no expected kind, use the head map with all actions
        |kind| self.kind_head_map.get(&kind).expect(INVALID_EXPECTED_KIND),
      );

      while let Some(((input_start, actions_len), (output, action_index, muted))) =
        ActionInput::new(text, options.start + digested, &mut *options.action_state).and_then(
          |mut input| {
            let actions = head_map.get(input.next());

            traverse_actions_mut(
              &mut input,
              actions,
              &options.base.re_lex,
              &mut re_lexable_factory,
            )
            .map(|res| ((input.start(), actions.len()), res))
          },
        )
      {
        if let Some(token) = process_output(output, muted, input_start, &mut digested, &mut errors)
        {
          return LexOutput {
            digested,
            token: Some(token),
            re_lexable: re_lexable_factory.into_stateless_re_lexable(
              input_start,
              actions_len,
              action_index,
            ),
            errors,
          };
        }

        // else, muted, continue
      }
    }

    // no more input or no accepted actions
    return LexOutput {
      digested,
      token: None,
      re_lexable: Default::default(),
      errors,
    };
  }
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
  // update digested, no matter the output is muted or not
  *digested += output.digested;

  // collect errors if any
  if let Some(err) = output.error {
    errors.update((err, create_range(start, output.digested)));
  }

  // if not muted, emit token
  (!muted).then(|| create_token(output.kind, start, output.digested))
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
