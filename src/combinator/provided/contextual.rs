use crate::{
  action::{Action, Context, Output},
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

/// Generate contextual combinators.
#[macro_export]
macro_rules! contextual {
  ($state:ty, $heap:ty) => {
    #[allow(dead_code)]
    mod _impl_contextual_combinators {
      use $crate::combinator::{Combinator, Contextual, Eat, Next, Take, Till};

      /// Contextual version of [`eat`](whitehole::combinator::eat).
      #[inline]
      pub const fn eat<T>(pattern: T) -> Combinator<Contextual<Eat<T>, $state, $heap>> {
        Combinator::new(Contextual::new(Eat::new(pattern)))
      }

      /// Contextual version of [`next`](whitehole::combinator::next).
      #[inline]
      pub const fn next<F: Fn(char) -> bool>(
        condition: F,
      ) -> Combinator<Contextual<Next<F>, $state, $heap>> {
        Combinator::new(Contextual::new(Next::new(condition)))
      }

      pub mod bytes {
        use $crate::combinator::{Combinator, Contextual, Next};

        /// Contextual version of [`next`](whitehole::combinator::next).
        #[inline]
        pub const fn next<F: Fn(u8) -> bool>(
          condition: F,
        ) -> Combinator<Contextual<Next<F>, $state, $heap>> {
          Combinator::new(Contextual::new(Next::new(condition)))
        }
      }

      /// Contextual version of [`take`](whitehole::combinator::take).
      #[inline]
      pub const fn take(n: usize) -> Combinator<Contextual<Take, $state, $heap>> {
        Combinator::new(Contextual::new(Take::new(n)))
      }

      /// Contextual version of [`till`](whitehole::combinator::till).
      #[inline]
      pub const fn till<T>(pattern: T) -> Combinator<Contextual<Till<T>, $state, $heap>> {
        Combinator::new(Contextual::new(Till::new(pattern)))
      }
    }
    pub use _impl_contextual_combinators::*;

    // TODO: recur, wrap
  };
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::instant::Instant;

  #[test]
  fn test_contextual() {
    contextual!(i32, i32);

    let action = eat('a');
    action.exec(
      &Instant::new("abc"),
      Context {
        state: &mut 0,
        heap: &mut 0,
      },
    );
  }
}
