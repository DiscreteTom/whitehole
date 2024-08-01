use super::{
  exec::{traverse_actions_mut, update_state},
  StatelessLexer, StatelessTrimOptions,
};
use crate::{
  lexer::{
    action::ActionInput,
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
    // there is no expectation when trimming, so the re-lex is meaningless.
    // use the default re-lex context as a placeholder
    let re_lex = ReLexContext::default();

    let mut digested = 0;
    let mut errors = options.base.errors_to;

    while let Some((input_start, (output, _action_index, muted))) =
      ActionInput::new(text, options.start + digested, &mut *options.action_state).and_then(
        |mut input| {
          // the literal map's muted map contains all the muted actions
          let actions = self.literal_map.muted_map().get(input.next());

          traverse_actions_mut(
            &mut input,
            actions,
            &re_lex,
            // there is no expectation when trimming, so the re-lex is meaningless.
            // use `()` as a placeholder
            &mut (),
          )
          .map(|res| (input.start(), res))
        },
      )
    {
      debug_assert!(muted, "all actions should be muted when trimming");

      update_state(
        output.digested,
        output.error,
        input_start,
        &mut digested,
        &mut errors,
      );
    }

    // no more input or no accepted actions
    TrimOutput { digested, errors }
  }
}
