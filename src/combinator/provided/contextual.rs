use crate::{
  action::{Action, Input, Output},
  instant::Instant,
};
use std::{fmt::Debug, marker::PhantomData};

// TODO: more comments
/// Overwrite original [`Action`]'s `State` and `Heap` with new ones.
pub struct Contextual<T, State, Heap> {
  pub action: T,
  _phantom: PhantomData<(State, Heap)>,
}

impl<T, State, Heap> Contextual<T, State, Heap> {
  /// Create a new instance.
  #[inline]
  pub const fn new(inner: T) -> Self {
    Self {
      action: inner,
      _phantom: PhantomData,
    }
  }
}

impl<T: Clone, State, Heap> Clone for Contextual<T, State, Heap> {
  #[inline]
  fn clone(&self) -> Self {
    Self {
      action: self.action.clone(),
      _phantom: PhantomData,
    }
  }
}

impl<T: Copy, State, Heap> Copy for Contextual<T, State, Heap> {}

impl<T: Debug, State, Heap> Debug for Contextual<T, State, Heap> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("Contextual").field(&self.action).finish()
  }
}

unsafe impl<T: Action<State: Default, Heap: Default>, State, Heap> Action
  for Contextual<T, State, Heap>
{
  type Text = T::Text;
  type State = State;
  type Heap = Heap;
  type Value = T::Value;

  #[inline]
  fn exec(
    &self,
    input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
  ) -> Option<Output<Self::Value>> {
    self.action.exec(Input {
      instant: input.instant,
      state: &mut Default::default(),
      heap: &mut Default::default(),
    })
  }
}

/// Generate contextual combinators.
#[macro_export]
macro_rules! contextual {
  ($state:ty, $heap:ty) => {
    #[allow(dead_code)]
    mod _impl_contextual_combinators {
      #[allow(unused_imports)]
      use super::*;
      use $crate::action::{Input, Output};
      use $crate::combinator::{Combinator, Contextual};
      use $crate::instant::Instant;

      /// Contextual version of [`eat`](whitehole::combinator::eat).
      #[inline]
      pub const fn eat<T>(
        pattern: T,
      ) -> Combinator<Contextual<$crate::combinator::Eat<T>, $state, $heap>> {
        Combinator::new(Contextual::new($crate::combinator::Eat::new(pattern)))
      }

      /// Contextual version of [`next`](whitehole::combinator::next).
      #[inline]
      pub const fn next<F: Fn(char) -> bool>(
        condition: F,
      ) -> Combinator<Contextual<$crate::combinator::Next<F>, $state, $heap>> {
        Combinator::new(Contextual::new($crate::combinator::Next::new(condition)))
      }

      /// Contextual version of [`take`](whitehole::combinator::take).
      #[inline]
      pub const fn take(
        n: usize,
      ) -> Combinator<Contextual<$crate::combinator::Take, $state, $heap>> {
        Combinator::new(Contextual::new($crate::combinator::Take::new(n)))
      }

      /// Contextual version of [`till`](whitehole::combinator::till).
      #[inline]
      pub const fn till<T>(
        pattern: T,
      ) -> Combinator<Contextual<$crate::combinator::Till<T>, $state, $heap>> {
        Combinator::new(Contextual::new($crate::combinator::Till::new(pattern)))
      }

      /// Contextual version of [`wrap_unchecked`](whitehole::combinator::wrap_unchecked).
      #[inline]
      pub const unsafe fn wrap_unchecked<
        Value,
        F: Fn(Input<&Instant<&str>, &mut $state, &mut $heap>) -> Option<Output<Value>>,
      >(
        f: F,
      ) -> Combinator<Contextual<$crate::combinator::WrapUnchecked<F>, $state, $heap>> {
        Combinator::new(Contextual::new($crate::combinator::WrapUnchecked::new(f)))
      }

      /// Contextual version of [`wrap`](whitehole::combinator::wrap).
      #[inline]
      pub const fn wrap<
        Value,
        F: Fn(Input<&Instant<&str>, &mut $state, &mut $heap>) -> Option<Output<Value>>,
      >(
        f: F,
      ) -> Combinator<Contextual<$crate::combinator::Wrap<F>, $state, $heap>> {
        Combinator::new(Contextual::new($crate::combinator::Wrap::new(f)))
      }

      pub mod bytes {
        use super::*;

        /// Contextual version of [`eat`](whitehole::combinator::bytes::eat).
        #[inline]
        pub const fn eat<T>(
          pattern: T,
        ) -> Combinator<Contextual<$crate::combinator::bytes::Eat<T>, $state, $heap>> {
          Combinator::new(Contextual::new($crate::combinator::bytes::Eat::new(
            pattern,
          )))
        }

        /// Contextual version of [`bytes::next`](whitehole::combinator::bytes::next).
        #[inline]
        pub const fn next<F: Fn(u8) -> bool>(
          condition: F,
        ) -> Combinator<Contextual<$crate::combinator::bytes::Next<F>, $state, $heap>> {
          Combinator::new(Contextual::new($crate::combinator::bytes::Next::new(
            condition,
          )))
        }

        /// Contextual version of [`take`](whitehole::combinator::bytes::take).
        #[inline]
        pub const fn take(
          n: usize,
        ) -> Combinator<Contextual<$crate::combinator::bytes::Take, $state, $heap>> {
          Combinator::new(Contextual::new($crate::combinator::bytes::Take::new(n)))
        }

        /// Contextual version of [`till`](whitehole::combinator::bytes::till).
        #[inline]
        pub const fn till<T>(
          pattern: T,
        ) -> Combinator<Contextual<$crate::combinator::bytes::Till<T>, $state, $heap>> {
          Combinator::new(Contextual::new($crate::combinator::bytes::Till::new(
            pattern,
          )))
        }

        /// Contextual version of [`bytes::wrap_unchecked`](whitehole::combinator::bytes::wrap_unchecked).
        #[inline]
        pub const unsafe fn wrap_unchecked<
          Value,
          F: Fn(Input<&Instant<&[u8]>, &mut $state, &mut $heap>) -> Option<Output<Value>>,
        >(
          f: F,
        ) -> Combinator<Contextual<$crate::combinator::bytes::WrapUnchecked<F>, $state, $heap>>
        {
          Combinator::new(Contextual::new(
            $crate::combinator::bytes::WrapUnchecked::new(f),
          ))
        }

        /// Contextual version of [`bytes::wrap`](whitehole::combinator::bytes::wrap).
        #[inline]
        pub const fn wrap<
          Value,
          F: Fn(Input<&Instant<&[u8]>, &mut $state, &mut $heap>) -> Option<Output<Value>>,
        >(
          f: F,
        ) -> Combinator<Contextual<$crate::combinator::bytes::Wrap<F>, $state, $heap>> {
          Combinator::new(Contextual::new($crate::combinator::bytes::Wrap::new(f)))
        }
      }
    }
    pub use _impl_contextual_combinators::*;

    // TODO: recur
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
    action.exec(Input {
      instant: &Instant::new("abc"),
      state: &mut 0,
      heap: &mut 0,
    });
  }
}
