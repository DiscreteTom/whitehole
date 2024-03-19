use super::{options::StatelessLexOptions, StatelessLexer};
use crate::lexer::{
  options::ReLexContext,
  output::LexOutput,
  token::{Token, TokenKindIdProvider},
};

// TODO: add comments and examples
pub struct StatelessReLexable<ActionState> {
  pub state: ActionState,
  pub context: ReLexContext,
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
    Kind: TokenKindIdProvider<Kind>,
    ActionState: Default,
  {
    let mut action_state = ActionState::default();
    (
      self.lex_with_options(text, &mut action_state, StatelessLexOptions::default()),
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
    Kind: TokenKindIdProvider<Kind>,
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
    options: impl Into<StatelessLexOptions<'expect_text, Kind>>,
  ) -> LexOutput<Token<'text, Kind, ErrorType>, ReLexContext>
  where
    Kind: TokenKindIdProvider<Kind>,
  {
    let options: StatelessLexOptions<_> = options.into();

    Self::execute_actions(
      options.expectation.kind.map_or(
        &self.head_map, // if no expected kind, use the head map with all actions
        |kind| {
          self
            .kind_head_map
            .get(&kind)
            // this must be `Some`, unless the user set the wrong possible kinds for actions
            .expect("expected kind should exists in some action's possible kinds")
        },
      ),
      options.fork,
      // the default ReLexContext will set `skip` and `action_index` to 0
      // which means this is not a re-lex
      options.re_lex.as_ref().unwrap_or(&ReLexContext::default()),
      text,
      options.start,
      action_state,
      &options.expectation,
    )
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
