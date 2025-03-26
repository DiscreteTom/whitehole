use crate::{
  action::{Action, Context, Output},
  combinator::Combinator,
  instant::Instant,
};
use std::{fmt::Debug, marker::PhantomData};

pub struct Contextual<T, State, Heap> {
  pub inner: T,
  _phantom: PhantomData<(State, Heap)>,
}

impl<T, State, Heap> Contextual<T, State, Heap> {
  pub const fn new(inner: T) -> Self {
    Self {
      inner,
      _phantom: PhantomData,
    }
  }
}

impl<T: Clone, State, Heap> Clone for Contextual<T, State, Heap> {
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
  // TODO: add new
  // TODO: better design? comments.
  // TODO: should this be a decorator?
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
