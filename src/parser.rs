mod builder;
mod snapshot;

use crate::{
  combinator::{Combinator, Input},
  node::Node,
};

pub use builder::*;
pub use snapshot::*;

pub struct Parser<'a, 'text, Kind, State = (), Heap = ()> {
  /// See [`Input::state`](crate::combinator::Input::state).
  /// You can mutate this directly if needed.
  pub state: State,
  /// See [`Input::heap`](crate::combinator::Input::heap).
  /// You can mutate this directly if needed.
  pub heap: Heap,

  /// See [`Self::text`].
  text: &'text str,
  /// See [`Self::rest`].
  rest: &'text str,
  /// See [`Self::entry`].
  entry: Combinator<'a, Kind, State, Heap>,
}

impl<'a, 'text, Kind, State, Heap> Parser<'a, 'text, Kind, State, Heap> {
  /// The entry combinator.
  pub const fn entry(&self) -> &Combinator<'a, Kind, State, Heap> {
    &self.entry
  }

  /// The whole input text.
  pub const fn text(&self) -> &'text str {
    self.text
  }

  /// How many bytes are already digested.
  pub const fn digested(&self) -> usize {
    // TODO: cache this
    self.text.len() - self.rest.len()
  }

  /// Get the undigested text.
  pub const fn rest(&self) -> &'text str {
    self.rest
  }

  /// Consume self, return a new instance with the same combinator and a new text.
  /// [`Self::digested`] and [`Self::state`] will be reset to default.
  /// [`Self::heap`] won't change.
  pub fn reload<'new_text>(self, text: &'new_text str) -> Parser<'a, 'new_text, Kind, State, Heap>
  where
    State: Default,
  {
    self.reload_with(State::default(), text)
  }

  /// Consume self, return a new instance with the same combinator, a new text and the given state.
  /// [`Self::heap`] won't change.
  pub fn reload_with<'new_text>(
    self,
    state: State,
    text: &'new_text str,
  ) -> Parser<'a, 'new_text, Kind, State, Heap> {
    Parser {
      entry: self.entry,
      heap: self.heap,
      state,
      text,
      rest: text,
    }
  }

  // /// Take a snapshot of the current [`Self::state`], [`Self::text`] and [`Self::digested`].
  // pub fn snapshot(&self) -> Snapshot<'text, State>
  // where
  //   State: Clone,
  // {
  //   Snapshot {
  //     state: self.state.clone(),
  //     text: self.text,
  //     rest: self.rest,
  //   }
  // }

  // /// Restore [`Self::state`], [`Self::text`] and [`Self::digested`] from a [`Snapshot`].
  // pub fn restore(&mut self, snapshot: Snapshot<'text, State>) {
  //   self.state = snapshot.state;
  //   self.text = snapshot.text;
  //   self.rest = snapshot.rest;
  // }

  // /// Digest the next `n` chars and set [`Self::state`] to the default.
  // ///
  // /// Usually when you digest some chars from outside of the parser
  // /// (e.g. by an error recovery strategy),
  // /// the state should be reset to the default.
  // /// If you want to keep the state, use [`Self::digest_with`] instead.
  // /// # Caveats
  // /// The caller should make sure `n` is no greater than the rest text length,
  // /// this will be checked using [`debug_assert`].
  // pub fn digest(&mut self, n: usize) -> &mut Self
  // where
  //   State: Default,
  // {
  //   self.digest_with(State::default(), n)
  // }

  // /// Digest the next `n` chars and optionally set [`Self::state`].
  // /// # Caveats
  // /// The caller should make sure `n` is no greater than the rest text length,
  // /// this will be checked using [`debug_assert`].
  // pub fn digest_with(&mut self, state: impl Into<Option<State>>, n: usize) -> &mut Self {
  //   debug_assert!(self.digested + n <= self.text.len());
  //   self.digested += n;
  //   if let Some(state) = state.into() {
  //     self.state = state;
  //   }
  //   self
  // }

  /// Try to yield the next [`Node`].
  /// Return [`None`] if the text is already fully digested
  /// or the combinator rejects.
  pub fn parse(&mut self) -> Option<Node<Kind>> {
    let output = self.entry.parse(&mut Input::new(
      &self.rest,
      self.digested(),
      &mut self.state,
      &mut self.heap,
    )?)?;
    let node = Node {
      kind: output.kind,
      range: self.digested()..output.rest.len() - self.rest.len() + self.digested(),
    };
    self.rest = output.rest;
    node.into()
  }

  // /// Try to yield the next [`Node`] without updating [`Self::digested`] and [`Self::state`].
  // /// [`Self::state`] will be cloned and returned.
  // /// Return [`None`] if the text is already fully digested
  // /// or the combinator rejects.
  // pub fn peek(&mut self) -> (Option<Node<Kind>>, State)
  // where
  //   State: Clone,
  // {
  //   let mut tmp_state = self.state.clone();
  //   (
  //     Input::new(self.text, self.digested, &mut tmp_state, &mut self.heap)
  //       .and_then(|mut input| self.entry.parse(&mut input))
  //       .map(|output| Node {
  //         kind: output.kind,
  //         range: self.digested..self.digested + output.digested,
  //       }),
  //     tmp_state,
  //   )
  // }
}
