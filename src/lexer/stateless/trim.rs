use super::{StatelessLexer, StatelessTrimOptions};
use crate::lexer::{
  action::ActionInput,
  output::TrimOutput,
  re_lex::ReLexContext,
  stateless::utils::{break_loop_on_none, traverse_actions},
};

impl<Kind, State> StatelessLexer<Kind, State> {
  /// Lex with muted actions, the default state and the default options.
  ///
  /// This function will create a new state and return it.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder};
  /// # let stateless = LexerBuilder::new().append(exact("2")).build_stateless();
  /// let (output, state) = stateless.trim("123");
  /// ```
  #[inline]
  pub fn trim<'text>(&self, text: &'text str) -> (TrimOutput, State)
  where
    State: Default,
  {
    let mut state = State::default();
    (self.trim_with(text, |o| o.state(&mut state)), state)
  }

  /// Lex with muted actions and the given options builder.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder};
  /// # let stateless = LexerBuilder::new().append(exact("2")).build_stateless();
  /// # let mut state = ();
  /// stateless.trim_with("123", |o| o.state(&mut state));
  /// ```
  #[inline]
  pub fn trim_with<'text, 'state>(
    &self,
    text: &'text str,
    options_builder: impl FnOnce(StatelessTrimOptions<()>) -> StatelessTrimOptions<&'state mut State>,
  ) -> TrimOutput
  where
    State: 'state,
  {
    self.trim_with_options(text, options_builder(StatelessTrimOptions::new()))
  }

  /// Lex with muted actions and the given [`StatelessTrimOptions`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder, stateless::StatelessTrimOptions};
  /// # let stateless = LexerBuilder::new().append(exact("2")).build_stateless();
  /// # let mut state = ();
  /// let options = StatelessTrimOptions::new().state(&mut state);
  /// stateless.trim_with_options("123", options);
  /// ```
  pub fn trim_with_options<'text, 'state>(
    &self,
    text: &'text str,
    options: StatelessTrimOptions<&'state mut State>,
  ) -> TrimOutput {
    // there is no expectation when trimming, so the fork is meaningless.
    // use the default re-lex context as a placeholder
    let re_lex = ReLexContext::default();

    let mut digested = 0;

    loop {
      let input_start = options.start + digested;
      let input = break_loop_on_none!(ActionInput::new(text, input_start, &mut *options.state));
      // the literal map's muted map contains all the muted actions
      let actions = self.literal_map.muted_map().get(input.next());
      let res = traverse_actions(input, actions, &re_lex);
      let (output, _action_index, muted) = break_loop_on_none!(res);

      debug_assert!(muted, "all actions should be muted when trimming");

      digested += output.digested;
    }

    // no more input or no accepted actions
    TrimOutput { digested }
  }
}
