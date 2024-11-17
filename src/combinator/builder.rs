use super::{eat, next, Combinator, Eat};
use crate::{parse::Parse, Combinator};
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, Default)]
pub struct Builder<State, Heap> {
  _phantom: PhantomData<(State, Heap)>,
}

impl<State, Heap> Builder<State, Heap> {
  #[inline]
  pub fn new() -> Self {
    Self {
      _phantom: PhantomData,
    }
  }

  #[inline]
  pub fn eat(&self, pattern: impl Eat<State, Heap>) -> Combinator!((), State, Heap) {
    eat(pattern)
  }

  #[inline]
  pub fn next(&self, condition: impl Fn(char) -> bool) -> Combinator!((), State, Heap) {
    next(condition)
  }
}
