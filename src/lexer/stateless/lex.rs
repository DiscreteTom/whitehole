use super::{
  head_map::{HeadMap, RuntimeActions},
  literal_map::LiteralMap,
  options::StatelessLexOptions,
  utils::{break_loop_on_none, lex, prepare_input},
  StatelessLexer,
};
use crate::{
  kind::SubKindId,
  lexer::{
    action::{ActionInput, ActionOutput},
    fork::{ForkOutputFactory, LexOptionsFork},
    output::LexOutput,
    stateless::utils::traverse_actions,
    token::Token,
  },
  utils::lookup::Lookup,
};

impl<'a, Kind, State, Heap> StatelessLexer<'a, Kind, State, Heap> {
  const INVALID_EXPECTED_KIND: &'static str = "no action is defined for the expected kind";
  const INVALID_EXPECTED_LITERAL: &'static str = "no action is defined for the expected literal";

  /// Lex from the start of the input text with the default state and options.
  ///
  /// This function will create a new state and a new heap and return them.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, builder::LexerBuilder};
  /// # let stateless = LexerBuilder::new().append(exact("1")).build_stateless();
  /// let (output, state, heap) = stateless.lex("123");
  /// ```
  #[inline]
  pub fn lex(&self, text: &str) -> (LexOutput<Token<Kind>, ()>, State, Heap)
  where
    State: Default,
    Heap: Default,
  {
    let mut state = State::default();
    let mut heap = Heap::default();
    (
      self.lex_with(text, |o| o.state(&mut state).heap(&mut heap)),
      state,
      heap,
    )
  }

  /// Lex with the given options builder.
  /// # Panics
  /// Panics if no action is defined for the expected kind or literal.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, builder::LexerBuilder};
  /// # let stateless = LexerBuilder::new().append(exact("2")).build_stateless();
  /// # let mut state = ();
  /// # let mut heap = ();
  /// stateless.lex_with("123", |o| o.state(&mut state).heap(&mut heap));
  /// ```
  #[inline]
  pub fn lex_with<'state, 'heap, Fork: LexOptionsFork>(
    &self,
    text: &str,
    options_builder: impl FnOnce(
      StatelessLexOptions<Kind, (), (), ()>,
    )
      -> StatelessLexOptions<Kind, &'state mut State, &'heap mut Heap, Fork>,
  ) -> LexOutput<Token<Kind>, <Fork::OutputFactoryType as ForkOutputFactory>::ForkOutputType>
  where
    State: 'state,
    Heap: 'heap,
  {
    self.lex_with_options(text, options_builder(StatelessLexOptions::new()))
  }

  /// Lex with the given [`StatelessLexOptions`].
  /// # Panics
  /// Panics if no action is defined for the expected kind or literal.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, builder::LexerBuilder, stateless::StatelessLexOptions};
  /// # let stateless = LexerBuilder::new().append(exact("2")).build_stateless();
  /// # let mut state = ();
  /// # let mut heap = ();
  /// let options = StatelessLexOptions::new().state(&mut state).heap(&mut heap);
  /// stateless.lex_with_options("123", options);
  /// ```
  pub fn lex_with_options<Fork: LexOptionsFork>(
    &self,
    text: &str,
    options: StatelessLexOptions<Kind, &mut State, &mut Heap, Fork>,
  ) -> LexOutput<Token<Kind>, <Fork::OutputFactoryType as ForkOutputFactory>::ForkOutputType> {
    let mut digested = 0;

    macro_rules! lex_with_actions {
      ($actions_getter:ident, $output_validator:ident) => {
        while let Some(mut input) = prepare_input!(text, digested, options) {
          let actions = $actions_getter!(input);
          let (output, action_index, muted) = lex!(input, actions, &options.base.re_lex, digested);

          if !muted {
            $output_validator!(output);

            return done_with_token(
              digested,
              create_token(&input, output),
              Fork::OutputFactoryType::default(),
              input.start(),
              actions.len(),
              action_index,
            );
          }

          // else, muted, continue
        }

        // no more input or no accepted actions
        return done_without_token(digested);
      };
    }

    macro_rules! validate_expected_kind {
      ($output:expr) => {
        debug_assert!(options
          .base
          .expectation
          .kind
          .map(|id| id == $output.binding.id())
          .unwrap_or(true));
      };
    }

    if let Some(literal) = options.base.expectation.literal {
      let (literal_map, head_map) =
        self.get_literal_head_map(options.base.expectation.kind, literal);

      macro_rules! actions_getter_with_literal {
        ($input:expr) => {
          get_actions_by_literal_map(&$input, literal, literal_map, head_map)
        };
      }

      macro_rules! validate_expected_kind_and_literal {
        ($output:expr) => {
          // we've already checked if the `input.rest()` starts with the `literal`
          // in `get_actions_by_literal_map`
          // so we only need to check whether the digested length is right
          debug_assert_eq!($output.digested, literal.len());
          validate_expected_kind!($output);
        };
      }

      lex_with_actions!(
        actions_getter_with_literal,
        validate_expected_kind_and_literal
      );
    } else {
      // else, no expected literal
      let head_map = self.get_kind_head_map(options.base.expectation.kind);

      macro_rules! actions_getter_without_literal {
        ($input:expr) => {
          head_map.get($input.next())
        };
      }

      lex_with_actions!(actions_getter_without_literal, validate_expected_kind);
    }
  }

  // there is no `StatelessLexer::peek()` because it is just the same with `StatelessLexer::lex()`

  /// Peek with the given options builder.
  /// This will clone [`StatelessLexOptions::state`] if it is mutated.
  /// # Panics
  /// Panics if no action is defined for the expected kind or literal.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, builder::LexerBuilder};
  /// # let stateless = LexerBuilder::new().append(exact("2")).build_stateless();
  /// # let state = ();
  /// # let mut heap = ();
  /// let (output, mutated_state) = stateless.peek_with("123", |o| o.state(&state).heap(&mut heap));
  /// ```
  #[inline]
  pub fn peek_with<'state, 'heap, Fork: LexOptionsFork>(
    &self,
    text: &str,
    options_builder: impl FnOnce(
      StatelessLexOptions<Kind, (), (), ()>,
    ) -> StatelessLexOptions<Kind, &'state State, &'heap mut Heap, Fork>,
  ) -> (
    LexOutput<Token<Kind>, <Fork::OutputFactoryType as ForkOutputFactory>::ForkOutputType>,
    State,
  )
  where
    State: Clone + 'state,
    Heap: 'heap,
  {
    self.peek_with_options(text, options_builder(StatelessLexOptions::new()))
  }

  /// Peek with the given [`StatelessLexOptions`].
  /// This will clone [`StatelessLexOptions::state`] if it is mutated.
  /// # Panics
  /// Panics if no action is defined for the expected kind or literal.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, builder::LexerBuilder, stateless::StatelessLexOptions};
  /// # let stateless = LexerBuilder::new().append(exact("2")).build_stateless();
  /// # let state = ();
  /// # let mut heap = ();
  /// let options = StatelessLexOptions::new().state(&state).heap(&mut heap);
  /// let (output, mutated_state) = stateless.peek_with_options("123", options);
  /// ```
  pub fn peek_with_options<Fork: LexOptionsFork>(
    &self,
    text: &str,
    options: StatelessLexOptions<Kind, &State, &mut Heap, Fork>,
  ) -> (
    LexOutput<Token<Kind>, <Fork::OutputFactoryType as ForkOutputFactory>::ForkOutputType>,
    State,
  )
  where
    State: Clone,
  {
    let mut state = options.state.clone();
    let output = self.lex_with_options(text, options.state(&mut state));
    (output, state)
  }

  fn get_literal_head_map(
    &self,
    kind: Option<SubKindId<Kind>>,
    literal: &str,
  ) -> (&LiteralMap<Kind, State, Heap>, &HeadMap<Kind, State, Heap>) {
    let literal_map = kind.map_or(&self.literal_map, |kind| {
      self
        .kind_literal_map
        .get(kind.value())
        .expect(Self::INVALID_EXPECTED_KIND)
    });
    let head_map = literal_map
      .known_map()
      .get(literal)
      .expect(Self::INVALID_EXPECTED_LITERAL);
    (literal_map, head_map)
  }

  fn get_kind_head_map(&self, kind: Option<SubKindId<Kind>>) -> &HeadMap<Kind, State, Heap> {
    kind.map_or(
      &self.head_map, // if no expected kind, use the head map with all actions
      |kind| {
        self
          .kind_head_map
          .get(kind.value())
          .expect(Self::INVALID_EXPECTED_KIND)
      },
    )
  }
}

fn create_token<Kind, State, Heap>(
  input: &ActionInput<&mut State, &mut Heap>,
  output: ActionOutput<Kind>,
) -> Token<Kind> {
  Token {
    binding: output.binding,
    // user is responsible to ensure the digested length is valid
    range: input.range_unchecked(output.digested),
  }
}

fn done_with_token<Kind, ForkOutputFactoryType: ForkOutputFactory>(
  digested: usize,
  token: Token<Kind>,
  fork_output_factory: ForkOutputFactoryType,
  input_start: usize,
  actions_len: usize,
  action_index: usize,
) -> LexOutput<Token<Kind>, ForkOutputFactoryType::ForkOutputType> {
  LexOutput {
    digested,
    token: Some(token),
    fork: fork_output_factory.into_fork_output(input_start, actions_len, action_index),
  }
}

fn get_actions_by_literal_map<'a, 'this: 'a, Kind, State, Heap>(
  input: &ActionInput<&mut State, &mut Heap>,
  literal: &str,
  literal_map: &'this LiteralMap<'a, Kind, State, Heap>,
  head_map: &'this HeadMap<'a, Kind, State, Heap>,
) -> &'this RuntimeActions<'a, Kind, State, Heap> {
  {
    if !input.rest().starts_with(literal) {
      // prefix mismatch, only execute muted actions
      literal_map.muted_map()
    } else {
      // prefix match, use the literal's head map
      head_map
    }
  }
  .get(input.next())
}

#[inline]
fn done_without_token<TokenType, ForkOutputType: Default>(
  digested: usize,
) -> LexOutput<TokenType, ForkOutputType> {
  LexOutput {
    digested,
    token: None,
    fork: ForkOutputType::default(),
  }
}

// TODO: add tests
// #[cfg(test)]
// mod tests {
//   use crate::lexer::{action::exact, builder::LexerBuilder};
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
