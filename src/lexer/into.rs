use super::{builder::LexerBuilder, stateless::StatelessLexer, Lexer};
use std::rc::Rc;

/// A helper trait to convert common types into a lexer.
pub trait IntoLexer<'a, Kind, State, Heap>: Sized {
  /// Consume self, build a [`Lexer`] with the provided `state` and `text`.
  fn into_lexer_with<'text>(
    self,
    state: State,
    heap: Heap,
    text: &'text str,
  ) -> Lexer<'a, 'text, Kind, State, Heap>;

  /// Consume self, build a [`Lexer`] with the provided `text` and the default `State` and `Heap`.
  #[inline]
  fn into_lexer<'text>(self, text: &'text str) -> Lexer<'a, 'text, Kind, State, Heap>
  where
    State: Default,
    Heap: Default,
  {
    self.into_lexer_with(State::default(), Heap::default(), text)
  }
}

impl<'a, Kind, State, Heap> IntoLexer<'a, Kind, State, Heap>
  for Rc<StatelessLexer<'a, Kind, State, Heap>>
{
  #[inline]
  fn into_lexer_with<'text>(
    self,
    state: State,
    heap: Heap,
    text: &'text str,
  ) -> Lexer<'a, 'text, Kind, State, Heap> {
    Lexer::new(self, state, heap, text)
  }
}

impl<'a, Kind, State, Heap> IntoLexer<'a, Kind, State, Heap>
  for StatelessLexer<'a, Kind, State, Heap>
{
  #[inline]
  fn into_lexer_with<'text>(
    self,
    state: State,
    heap: Heap,
    text: &'text str,
  ) -> Lexer<'a, 'text, Kind, State, Heap> {
    Rc::new(self).into_lexer_with(state, heap, text)
  }
}

impl<'a, Kind, State, Heap> IntoLexer<'a, Kind, State, Heap>
  for LexerBuilder<'a, Kind, State, Heap>
{
  #[inline]
  fn into_lexer_with<'text>(
    self,
    state: State,
    heap: Heap,
    text: &'text str,
  ) -> Lexer<'a, 'text, Kind, State, Heap> {
    self.build_stateless().into_lexer_with(state, heap, text)
  }
}
