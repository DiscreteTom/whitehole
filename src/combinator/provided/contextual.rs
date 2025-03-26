use crate::{
  action::{Action, Context, Output},
  combinator::Combinator,
  instant::Instant,
};
use std::{fmt::Debug, marker::PhantomData};

// TODO: more comments
/// Overwrite original [`Action`]'s `State` and `Heap` with new ones.
pub struct Contextual<T, State, Heap> {
  pub inner: T,
  _phantom: PhantomData<(State, Heap)>,
}

impl<T, State, Heap> Contextual<T, State, Heap> {
  /// Create a new instance.
  #[inline]
  pub const fn new(inner: T) -> Self {
    Self {
      inner,
      _phantom: PhantomData,
    }
  }
}

impl<T: Clone, State, Heap> Clone for Contextual<T, State, Heap> {
  #[inline]
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
      _phantom: PhantomData,
    }
  }
}

impl<T: Copy, State, Heap> Copy for Contextual<T, State, Heap> {}

impl<T: Debug, State, Heap> Debug for Contextual<T, State, Heap> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("Contextual").field(&self.inner).finish()
  }
}

impl<T> Combinator<T> {
  // TODO: better design? comments.
  // TODO: should this be a decorator?
  #[inline]
  pub fn with_ctx<State, Heap>(self) -> Combinator<Contextual<T, State, Heap>> {
    Combinator::new(Contextual::new(self.action))
  }
}

unsafe impl<Text: ?Sized, T: Action<Text, State: Default, Heap: Default>, State, Heap> Action<Text>
  for Contextual<T, State, Heap>
{
  type Value = T::Value;
  type State = State;
  type Heap = Heap;

  #[inline]
  fn exec(
    &self,
    instant: &Instant<&Text>,
    _ctx: Context<&mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    self.inner.exec(
      instant,
      Context {
        state: &mut Default::default(),
        heap: &mut Default::default(),
      },
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{combinator::eat, instant::Instant};

  #[test]
  fn test_contextual() {
    let action = eat('a').with_ctx::<i32, i32>();
    action.exec(
      &Instant::new("abc"),
      Context {
        state: &mut 0,
        heap: &mut 0,
      },
    );
  }
}
