use super::{common::Validator, StatelessLexer};
use crate::lexer::{
  expectation::Expectation,
  options::ReLexContext,
  output::LexOutput,
  stateless::common::OutputHandler,
  token::{Token, TokenKind},
};

pub struct StatelessLexOptions<'expect_text, Kind> {
  pub expectation: Expectation<'expect_text, Kind>,
  /// The start index of the text to lex.
  pub start: usize,
  /// Provide this if the lex is a re-lex.
  pub re_lex: Option<ReLexContext>,
}

impl<'expect_text, Kind> Default for StatelessLexOptions<'expect_text, Kind> {
  fn default() -> Self {
    Self {
      start: 0,
      expectation: Expectation::default(),
      re_lex: None,
    }
  }
}

impl<'expect_text, Kind> StatelessLexOptions<'expect_text, Kind> {
  /// The start index of the text to lex.
  pub fn start(mut self, start: usize) -> Self {
    self.start = start;
    self
  }
  pub fn expect(mut self, expectation: impl Into<Expectation<'expect_text, Kind>>) -> Self {
    self.expectation = expectation.into();
    self
  }
  /// Provide this if the lex is a re-lex.
  pub fn re_lex(mut self, re_lex: ReLexContext) -> Self {
    self.re_lex = Some(re_lex);
    self
  }
}

impl<Kind, ActionState, ErrorType> StatelessLexer<Kind, ActionState, ErrorType> {
  /// Lex with the default action state and the default [`StatelessLexOptions`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)] enum MyKind { A }
  /// # let stateless = LexerBuilder::<MyKind>::new().define(MyKind::A, exact("1")).build_stateless();
  /// stateless.lex("123");
  /// ```
  pub fn lex<'text>(
    &self,
    text: &'text str,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexContext>,
    ActionState,
  )
  where
    Kind: TokenKind<Kind>,
    ActionState: Default,
  {
    let mut action_state = ActionState::default();
    (
      self.lex_with_options(text, &mut action_state, StatelessLexOptions::default()),
      action_state,
    )
  }

  /// Lex with the default action state and the given [`StatelessLexOptions`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)] enum MyKind { A }
  /// # let stateless = LexerBuilder::<MyKind>::new().define(MyKind::A, exact("2")).build_stateless();
  /// stateless.lex_with_default("123", |o| o.start(1));
  /// ```
  pub fn lex_with_default<'text, 'expect_text>(
    &self,
    text: &'text str,
    options_builder: impl FnOnce(
      StatelessLexOptions<'expect_text, Kind>,
    ) -> StatelessLexOptions<'expect_text, Kind>,
  ) -> (
    LexOutput<Token<'text, Kind, ErrorType>, ReLexContext>,
    ActionState,
  )
  where
    Kind: TokenKind<Kind>,
    ActionState: Default,
  {
    let mut action_state = ActionState::default();
    (
      self.lex_with(text, &mut action_state, options_builder),
      action_state,
    )
  }

  /// Lex with the given action state and the given [`StatelessLexOptions`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)] enum MyKind { A }
  /// # let stateless = LexerBuilder::<MyKind>::new().define(MyKind::A, exact("2")).build_stateless();
  /// # let mut action_state = ();
  /// stateless.lex_with("123", &mut action_state, |o| o.start(1));
  /// ```
  pub fn lex_with<'text, 'expect_text>(
    &self,
    text: &'text str,
    action_state: &mut ActionState,
    options_builder: impl FnOnce(
      StatelessLexOptions<'expect_text, Kind>,
    ) -> StatelessLexOptions<'expect_text, Kind>,
  ) -> LexOutput<Token<'text, Kind, ErrorType>, ReLexContext>
  where
    Kind: TokenKind<Kind>,
  {
    self.lex_with_options(
      text,
      action_state,
      options_builder(StatelessLexOptions::default()),
    )
  }

  /// Lex with the given action state and the given [`StatelessLexOptions`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder, stateless::StatelessLexOptions};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)] enum MyKind { A }
  /// # let stateless = LexerBuilder::<MyKind>::new().define(MyKind::A, exact("2")).build_stateless();
  /// # let mut action_state = ();
  /// # let options = StatelessLexOptions::default();
  /// stateless.lex_with_options("123", &mut action_state, options);
  /// ```
  pub fn lex_with_options<'text, 'expect_text>(
    &self,
    text: &'text str,
    action_state: &mut ActionState,
    options: StatelessLexOptions<'expect_text, Kind>,
  ) -> LexOutput<Token<'text, Kind, ErrorType>, ReLexContext>
  where
    Kind: TokenKind<Kind>,
  {
    // use static to avoid allocation in each call
    static OUTPUT_HANDLER: OutputHandler = OutputHandler {
      update_lex_output: true,
      create_token: true,
    };

    let Expectation {
      kind: exp_kind,
      text: exp_text,
    } = options.expectation;

    Self::execute_actions(
      exp_kind.map_or(
        // if no expected kind, use the head map with all actions
        &self.head_map,
        |kind| {
          self
            .kind_head_map
            .get(&kind)
            // this must be `Some`, unless the user set the wrong possible kinds for actions
            .expect("expected kind should exists in some action's possible kinds")
        },
      ),
      // the default ReLexContext will set `skip` and `action_index` to 0
      // which means this is not a re-lex
      options.re_lex.unwrap_or(ReLexContext::default()),
      move |input| {
        let text_mismatch = exp_text.is_some_and(|text| !input.rest().starts_with(text));
        Validator {
          // since we already filtered actions, we only need to skip actions
          // which are never muted and text mismatch
          skip_before_exec: Box::new(move |action| action.never_muted() && text_mismatch),
          accept_after_exec: Box::new(move |input, output| {
            output.muted
              || (
                // ensure expectation match.
                // we still need to check the kind after exec
                // because muted actions may yield unexpected kinds
                exp_kind.map_or(true, |kind| output.kind.id() == kind)
                  && exp_text.map_or(true, |text| &input.rest()[..output.digested] == text)
              )
          }),
        }
      },
      text,
      options.start,
      action_state,
      &OUTPUT_HANDLER,
    )
  }
}
