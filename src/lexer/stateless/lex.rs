use super::{options::StatelessLexOptions, StatelessLexer};
use crate::lexer::{
  action::ActionInput,
  fork::{ForkDisabled, LexOptionsFork},
  output::LexOutput,
  re_lex::ReLexableFactory,
  token::{Token, TokenKindIdProvider},
};

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
  ) -> (LexOutput<Token<'text, Kind, ErrorType>, ()>, ActionState)
  where
    Kind: TokenKindIdProvider<Kind>,
    ActionState: Default,
  {
    let mut action_state = ActionState::default();
    (
      self.lex_with_options(
        text,
        StatelessLexOptions::default().action_state(&mut action_state),
      ),
      action_state,
    )
  }

  /// Lex with the given  [`StatelessLexOptions`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)] enum MyKind { A }
  /// # let stateless = LexerBuilder::<MyKind>::new().define(MyKind::A, exact("2")).build_stateless();
  /// # let mut action_state = ();
  /// stateless.lex_with("123", |o| o.action_state(&mut action_state));
  /// ```
  pub fn lex_with<
    'text,
    'expect_text,
    'action_state,
    Fork: LexOptionsFork<'text, Kind, ActionState, ErrorType>,
  >(
    &self,
    text: &'text str,
    options_builder: impl FnOnce(
      StatelessLexOptions<'expect_text, Kind, (), ForkDisabled>,
    ) -> StatelessLexOptions<
      'expect_text,
      Kind,
      &'action_state mut ActionState,
      Fork,
    >,
  ) -> LexOutput<Token<'text, Kind, ErrorType>,  <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, ActionState, ErrorType>>::StatelessReLexableType>
  where
    Kind: TokenKindIdProvider<Kind>,
    ActionState: 'action_state,
  {
    self.lex_with_options(text, options_builder(StatelessLexOptions::default()))
  }

  /// Lex with the given [`StatelessLexOptions`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder, stateless::StatelessLexOptions};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)] enum MyKind { A }
  /// # let stateless = LexerBuilder::<MyKind>::new().define(MyKind::A, exact("2")).build_stateless();
  /// # let mut action_state = ();
  /// let options = StatelessLexOptions::default().action_state(&mut action_state);
  /// stateless.lex_with_options("123", options);
  /// ```
  pub fn lex_with_options<
    'text,
    'expect_text,
    'action_state,
    Fork: LexOptionsFork<'text, Kind, ActionState, ErrorType>,
  >(
    &self,
    text: &'text str,
    options: impl Into<StatelessLexOptions<'expect_text, Kind, &'action_state mut ActionState, Fork>>,
  ) -> LexOutput<Token<'text, Kind, ErrorType>, <Fork::ReLexableFactoryType as ReLexableFactory<'text, Kind, ActionState, ErrorType>>::StatelessReLexableType>
  where
    Kind: TokenKindIdProvider<Kind>,
    ActionState: 'action_state,
  {
    let options: StatelessLexOptions<_, _, _> = options.into();

    if let Some(literal) = options.base.expectation.literal {
      let literal_map = options
        .base
        .expectation
        .kind
        .map_or(&self.literal_map, |kind| {
          self
            .kind_literal_map
            .get(&kind)
            .expect("expected kind should exists in an action's kind")
        });
      let literal_map_item = literal_map
        .known_map()
        .get(literal)
        .expect("expected literal should exists in an action's literal");

      Self::execute_actions(
        |input: &ActionInput<ActionState>| {
          let literal_mismatch = !input.rest().starts_with(literal);
          if literal_mismatch {
            &literal_map_item.muted_head_map
          } else {
            &literal_map_item.head_map
          }
        },
        &options.base.re_lex,
        text,
        options.start,
        options.action_state,
        Fork::ReLexableFactoryType::default(),
      )
    } else {
      let head_map = options.base.expectation.kind.map_or(
        &self.head_map, // if no expected kind, use the head map with all actions
        |kind| {
          self
            .kind_head_map
            .get(&kind)
            // this must be `Some`, unless the user set the wrong possible kinds for actions
            .expect("expected kind should exists in some action's possible kinds")
        },
      );

      Self::execute_actions(
        |_| head_map,
        &options.base.re_lex,
        text,
        options.start,
        options.action_state,
        Fork::ReLexableFactoryType::default(),
      )
    }
  }
}

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
