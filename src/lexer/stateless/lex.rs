use super::{common::Validator, StatelessLexer};
use crate::lexer::{
  expectation::Expectation,
  options::{LexOptions, ReLexContext},
  output::LexOutput,
  stateless::common::UnMutedOutputHandler,
  token::{Token, TokenKind},
};
use std::ops::{Deref, DerefMut};

pub struct StatelessLexOptions<'expect_text, Kind> {
  /// The start index of the text to lex.
  pub start: usize,
  pub base: LexOptions<'expect_text, Kind>,
}

impl<'expect_text, Kind> Default for StatelessLexOptions<'expect_text, Kind> {
  fn default() -> Self {
    Self {
      start: 0,
      base: LexOptions::default(),
    }
  }
}

impl<'expect_text, Kind> From<Expectation<'expect_text, Kind>>
  for StatelessLexOptions<'expect_text, Kind>
{
  fn from(expectation: Expectation<'expect_text, Kind>) -> Self {
    Self {
      start: 0,
      base: expectation.into(),
    }
  }
}

impl<'expect_text, Kind> From<ReLexContext> for StatelessLexOptions<'expect_text, Kind> {
  fn from(re_lex: ReLexContext) -> Self {
    Self {
      start: 0,
      base: re_lex.into(),
    }
  }
}

impl<'expect_text, Kind> From<LexOptions<'expect_text, Kind>>
  for StatelessLexOptions<'expect_text, Kind>
{
  fn from(options: LexOptions<'expect_text, Kind>) -> Self {
    Self {
      start: 0,
      base: options,
    }
  }
}

impl<'expect_text, Kind> DerefMut for StatelessLexOptions<'expect_text, Kind> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.base
  }
}

impl<'expect_text, Kind> Deref for StatelessLexOptions<'expect_text, Kind> {
  type Target = LexOptions<'expect_text, Kind>;

  fn deref(&self) -> &Self::Target {
    &self.base
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
  /// If set, the [`LexOutput::re_lex`](crate::lexer::output::LexOutput::re_lex) might be `Some`.
  pub fn fork(mut self) -> Self {
    self.fork = true;
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
    static OUTPUT_HANDLER: UnMutedOutputHandler = UnMutedOutputHandler {
      update_lex_output: true,
      create_token: true,
    };

    let Expectation {
      kind: exp_kind,
      text: exp_text,
    } = options.expectation;

    Self::execute_actions(
      exp_kind.map_or(
        &self.head_map, // if no expected kind, use the head map with all actions
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
      &options.base.re_lex.unwrap_or(ReLexContext::default()),
      move |input| {
        // pre-calc and cache the text mismatch
        let text_mismatch = exp_text.is_some_and(|text| !input.rest().starts_with(text));
        Validator {
          // since we already filtered actions, we don't need to skip actions by kinds,
          // we only need to check text mismatch and maybe muted
          skip_before_exec: if text_mismatch {
            // text mismatch, only muted actions should be executed
            // so we skip never muted actions
            Box::new(|action| action.never_muted())
          } else {
            // text match, we shouldn't skip any action
            Box::new(|_| false)
          },
          accept_after_exec: Box::new(move |input, output| {
            // muted output is always accepted regardless of the expectation
            output.muted
              || (
                // ensure expectation match.
                // we still need to check the kind after exec
                // because maybe_muted actions may yield unexpected kinds and actually not muted
                exp_kind.map_or(true, |kind| output.kind.id() == kind)
                // same as the text, maybe_muted actions may accept unexpected text and actually not muted
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

#[cfg(test)]
mod tests {
  use crate::lexer::{action::exact, LexerBuilder};
  use whitehole_macros::_TokenKind;
  use MyKind::*;

  #[derive(_TokenKind, Clone)]
  enum MyKind {
    A,
    B,
  }

  #[test]
  #[should_panic]
  fn stateless_lexer_lex_with_unknown_kind() {
    let stateless = LexerBuilder::<MyKind>::new()
      .define(A, exact("A"))
      .build_stateless();
    stateless.lex_with("A", &mut (), |o| o.expect(B));
  }
}
