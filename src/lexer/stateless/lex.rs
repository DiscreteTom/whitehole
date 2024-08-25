use super::{
  head_map::{HeadMap, RuntimeActions},
  literal_map::LiteralMap,
  options::StatelessLexOptions,
  utils::{break_loop_on_none, extract_token},
  StatelessLexer,
};
use crate::{
  lexer::{
    action::{ActionInput, ActionOutput},
    fork::{ForkOutputFactory, LexOptionsFork},
    output::LexOutput,
    re_lex::ReLexContext,
    stateless::utils::traverse_actions,
    token::{Token, TokenKindId},
  },
  utils::lookup::lookup::Lookup,
};

impl<Kind, State, Heap> StatelessLexer<Kind, State, Heap> {
  const INVALID_EXPECTED_KIND: &'static str = "no action is defined for the expected kind";
  const INVALID_EXPECTED_LITERAL: &'static str = "no action is defined for the expected literal";

  /// Lex from the start of the input text with the default state and options.
  ///
  /// This function will create a new state and a new heap and return them.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder};
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
  /// # use whitehole::lexer::{action::exact, LexerBuilder};
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
  /// # use whitehole::lexer::{action::exact, LexerBuilder, stateless::StatelessLexOptions};
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
    if let Some(literal) = options.base.expectation.literal {
      let (literal_map, head_map) =
        self.get_literal_head_map(options.base.expectation.kind, literal);

      self.lex_with_literal(
        literal_map,
        head_map,
        0,
        options.start,
        text,
        options.state,
        options.heap,
        literal,
        &options.base.re_lex,
        Fork::OutputFactoryType::default(),
      )
    } else {
      // else, no expected literal
      let head_map = self.get_kind_head_map(options.base.expectation.kind);

      self.lex_without_literal(
        head_map,
        0,
        options.start,
        text,
        options.state,
        options.heap,
        &options.base.re_lex,
        Fork::OutputFactoryType::default(),
      )
    }
  }

  // there is no `StatelessLexer::peek()` because it is just the same with `StatelessLexer::lex()`

  /// Peek with the given options builder.
  /// This will clone [`StatelessLexOptions::state`] if it is mutated.
  /// # Panics
  /// Panics if no action is defined for the expected kind or literal.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::exact, LexerBuilder};
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
  /// # use whitehole::lexer::{action::exact, LexerBuilder, stateless::StatelessLexOptions};
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
    kind: Option<TokenKindId<Kind>>,
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

  fn get_kind_head_map(&self, kind: Option<TokenKindId<Kind>>) -> &HeadMap<Kind, State, Heap> {
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

  fn lex_with_literal<ForkOutputFactoryType: ForkOutputFactory>(
    &self,
    literal_map: &LiteralMap<Kind, State, Heap>,
    head_map: &HeadMap<Kind, State, Heap>,
    mut digested: usize,
    start: usize,
    text: &str,
    state: &mut State,
    heap: &mut Heap,
    literal: &str,
    re_lex: &ReLexContext,
    fork_output_factory: ForkOutputFactoryType,
  ) -> LexOutput<Token<Kind>, ForkOutputFactoryType::ForkOutputType> {
    loop {
      let input_start = start + digested;
      let input = break_loop_on_none!(ActionInput::new(text, input_start, &mut *state, &mut *heap));
      let actions = get_actions_by_literal_map(&input, literal, literal_map, head_map);
      let res = traverse_actions(input, actions, re_lex);
      let (output, action_index, muted) = break_loop_on_none!(res);

      if let Some(token) = process_output(output, muted, input_start, &mut digested) {
        return done_with_token(
          digested,
          token,
          fork_output_factory,
          input_start,
          actions.len(),
          action_index,
        );
      }

      // else, muted, continue
    }

    // no more input or no accepted actions
    return done_without_token(digested);
  }

  fn lex_without_literal<ForkOutputFactoryType: ForkOutputFactory>(
    &self,
    head_map: &HeadMap<Kind, State, Heap>,
    mut digested: usize,
    start: usize,
    text: &str,
    state: &mut State,
    heap: &mut Heap,
    re_lex: &ReLexContext,
    fork_output_factory: ForkOutputFactoryType,
  ) -> LexOutput<Token<Kind>, ForkOutputFactoryType::ForkOutputType> {
    loop {
      let input_start = start + digested;
      let input = break_loop_on_none!(ActionInput::new(text, input_start, &mut *state, &mut *heap));
      let actions = head_map.get(input.next());
      let res = traverse_actions(input, actions, re_lex);
      let (output, action_index, muted) = break_loop_on_none!(res);

      if let Some(token) = process_output(output, muted, input_start, &mut digested) {
        return done_with_token(
          digested,
          token,
          fork_output_factory,
          input_start,
          actions.len(),
          action_index,
        );
      }

      // else, muted, continue
    }

    // no more input or no accepted actions
    return done_without_token(digested);
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

fn get_actions_by_literal_map<'this, Kind, State, Heap>(
  input: &ActionInput<&mut State, &mut Heap>,
  literal: &str,
  literal_map: &'this LiteralMap<Kind, State, Heap>,
  head_map: &'this HeadMap<Kind, State, Heap>,
) -> &'this RuntimeActions<Kind, State, Heap> {
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

/// Process the output, update the digested, collect errors, and emit token if not muted.
/// Return the token if not muted, otherwise return [`None`].
fn process_output<Kind>(
  output: ActionOutput<Kind>,
  muted: bool,
  start: usize,
  digested: &mut usize,
) -> Option<Token<Kind>> {
  *digested += output.digested;
  extract_token(output.binding, output.digested, muted, start)
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
