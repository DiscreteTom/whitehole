use super::{eat, eat_unchecked, eater, eater_unchecked, next, till, wrap, Eat, Till};
use crate::{
  parse::{Input, Output},
  Combinator,
};
use std::marker::PhantomData;

/// A util struct to carry the `State` and `Heap` generic types
/// for a better type inference.
#[derive(Debug, Clone, Copy, Default)]
pub struct Builder<State, Heap> {
  _phantom: PhantomData<(State, Heap)>,
}

impl<State, Heap> Builder<State, Heap> {
  /// Create a new instance.
  #[inline]
  pub fn new() -> Self {
    Self {
      _phantom: PhantomData,
    }
  }

  /// See [`eat`].
  #[inline]
  pub fn eat(&self, pattern: impl Eat) -> Combinator!((), State, Heap) {
    eat(pattern)
  }

  /// See [`eat_unchecked`].
  #[allow(clippy::missing_safety_doc)]
  #[inline]
  pub unsafe fn eat_unchecked(&self, n: usize) -> Combinator!((), State, Heap) {
    eat_unchecked(n)
  }

  /// See [`eater`].
  #[inline]
  pub fn eater(
    &self,
    f: impl Fn(&mut Input<&mut State, &mut Heap>) -> usize,
  ) -> Combinator!((), State, Heap) {
    eater(f)
  }

  /// See [`eater_unchecked`].
  #[allow(clippy::missing_safety_doc)]
  #[inline]
  pub unsafe fn eater_unchecked(
    &self,
    f: impl Fn(&mut Input<&mut State, &mut Heap>) -> usize,
  ) -> Combinator!((), State, Heap) {
    eater_unchecked(f)
  }

  /// See [`next`].
  #[inline]
  pub fn next(&self, condition: impl Fn(char) -> bool) -> Combinator!((), State, Heap) {
    next(condition)
  }

  /// See [`till`].
  #[inline]
  pub fn till(&self, pattern: impl Till) -> Combinator!((), State, Heap) {
    till(pattern)
  }

  /// See [`wrap`].
  #[inline]
  pub fn wrap<Kind>(
    &self,
    parse: impl for<'text> Fn(&mut Input<'text, &mut State, &mut Heap>) -> Option<Output<'text, Kind>>,
  ) -> Combinator!(Kind, State, Heap) {
    wrap(parse)
  }
}

// TODO: tests
