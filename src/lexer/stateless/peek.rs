use super::{
  exec::{extract_token, update_state},
  options::StatelessLexOptions,
  StatelessLexer,
};
use crate::{
  lexer::{
    action::{ActionInput, ActionOutput},
    fork::LexOptionsFork,
    output::LexOutput,
    re_lex::ReLexableFactory,
    stateless::exec::traverse_actions,
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
    // TODO: move to impl scope
    const INVALID_EXPECTED_KIND: &str = "no action is defined for the expected kind";

    let mut digested = 0;
    let mut errors = options.base.errors_to;
    let mut re_lexable_factory = Fork::ReLexableFactoryType::default();
    let mut new_action_state = None;

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
        ActionInput::new(text, options.start + digested, options.action_state).and_then(|input| {
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

          let (output, action_state) = traverse_actions(
            &input,
            actions,
            &options.base.re_lex,
            &mut re_lexable_factory,
          );
          new_action_state = action_state;
          output.map(|res| ((input.start(), actions.len()), res))
        })
      {
        if let Some(token) = process_output(output, muted, input_start, &mut digested, &mut errors)
        {
          return (
            LexOutput {
              digested,
              token: Some(token),
              re_lexable: re_lexable_factory.into_stateless_re_lexable(
                input_start,
                actions_len,
                action_index,
              ),
              errors,
            },
            new_action_state,
          );
        }

        // if action state is mutated, use lex
        if let Some(action_state) = new_action_state {
          todo!() // TODO
        }

        // else, muted, continue
      }
    } else {
      // // else, no expected literal
      // let head_map = options.base.expectation.kind.map_or(
      //   &self.head_map, // if no expected kind, use the head map with all actions
      //   |kind| self.kind_head_map.get(&kind).expect(INVALID_EXPECTED_KIND),
      // );

      // while let Some(((input_start, actions_len), (output, action_index, muted))) =
      //   ActionInput::new(text, options.start + digested, &mut *options.action_state).and_then(
      //     |mut input| {
      //       let actions = head_map.get(input.next());

      //       traverse_actions_mut(
      //         &mut input,
      //         actions,
      //         &options.base.re_lex,
      //         &mut re_lexable_factory,
      //       )
      //       .map(|res| ((input.start(), actions.len()), res))
      //     },
      //   )
      // {
      //   if let Some(token) = process_output(output, muted, input_start, &mut digested, &mut errors)
      //   {
      //     return LexOutput {
      //       digested,
      //       token: Some(token),
      //       re_lexable: re_lexable_factory.into_stateless_re_lexable(
      //         input_start,
      //         actions_len,
      //         action_index,
      //       ),
      //       errors,
      //     };
      //   }

      //   // else, muted, continue
      // }
    }

    // no more input or no accepted actions
    return (
      LexOutput {
        digested,
        token: None,
        re_lexable: Default::default(),
        errors,
      },
      new_action_state,
    );
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
  update_state(output.digested, output.error, start, digested, errors);
  extract_token(output.kind, output.digested, muted, start)
}
