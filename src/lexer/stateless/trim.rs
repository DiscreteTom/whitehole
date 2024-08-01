use super::{
  exec::{create_range, traverse_actions_mut},
  StatelessLexer, StatelessTrimOptions,
};
use crate::{
  lexer::{
    action::{ActionInput, ActionOutput},
    output::TrimOutput,
    re_lex::ReLexContext,
    token::{Range, TokenKindIdProvider},
  },
  utils::Accumulator,
};

impl<Kind, ActionState, ErrorType> StatelessLexer<Kind, ActionState, ErrorType> {
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
    Kind: TokenKindIdProvider<TokenKind = Kind>,
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
    Kind: TokenKindIdProvider<TokenKind = Kind>,
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
    Kind: TokenKindIdProvider<TokenKind = Kind>,
  {
    let re_lex = ReLexContext::default();

    let mut digested = 0;
    let mut errors = options.base.errors_to;

    while let Some((input_start, (output, _action_index, _muted))) =
      ActionInput::new(text, options.start + digested, &mut *options.action_state).and_then(
        |mut input| {
          // the literal map's muted map contains all the muted actions
          let actions = self.literal_map.muted_map().get(input.next());

          traverse_actions_mut(&mut input, actions, &re_lex, &mut ())
            .map(|res| (input.start(), res))
        },
      )
    {
      process_output(output, input_start, &mut digested, &mut errors);
    }

    TrimOutput { digested, errors }
  }
}

/// Process the output, update the digested, collect errors.
fn process_output<Kind, ErrorType, ErrAcc: Accumulator<(ErrorType, Range)>>(
  output: ActionOutput<Kind, Option<ErrorType>>,
  start: usize,
  digested: &mut usize,
  errors: &mut ErrAcc,
) {
  // update digested, no matter the output is muted or not
  *digested += output.digested;

  // collect errors if any
  if let Some(err) = output.error {
    errors.update((err, create_range(start, output.digested)));
  }
}
