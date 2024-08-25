use super::{StatelessLexer, StatelessTrimOptions};
use crate::lexer::{
  action::ActionInput,
  output::TrimOutput,
  re_lex::ReLexContext,
  stateless::utils::{break_loop_on_none, traverse_actions},
};

impl<Kind, State, Heap> StatelessLexer<Kind, State, Heap> {
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
  pub fn trim<'text>(&self, text: &'text str) -> (TrimOutput, State, Heap)
  where
    State: Default,
    Heap: Default,
  {
    let mut state = State::default();
    let mut heap = Heap::default();
    (
      self.trim_with(text, |o| o.state(&mut state).heap(&mut heap)),
      state,
      heap,
    )
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
  pub fn trim_with<'text, 'state, 'heap>(
    &self,
    text: &'text str,
    options_builder: impl FnOnce(
      StatelessTrimOptions<(), ()>,
    ) -> StatelessTrimOptions<&'state mut State, &'heap mut Heap>,
  ) -> TrimOutput
  where
    State: 'state,
    Heap: 'heap,
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
  pub fn trim_with_options<'text>(
    &self,
    text: &'text str,
    options: StatelessTrimOptions<&mut State, &mut Heap>,
  ) -> TrimOutput {
    // there is no expectation when trimming, so the fork is meaningless.
    // use the default re-lex context as a placeholder
    let re_lex = ReLexContext::default();

    let mut digested = 0;

    loop {
      let input_start = options.start + digested;
      let input = break_loop_on_none!(ActionInput::new(
        text,
        input_start,
        &mut *options.state,
        &mut *options.heap
      ));
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
