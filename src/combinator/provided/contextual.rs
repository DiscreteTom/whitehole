use crate::{
  action::{Action, Input, Output},
  instant::Instant,
};
use std::{fmt::Debug, marker::PhantomData};

/// Provide context information (`State` and `Heap`) to the original non-contextual action.
///
/// See [`contextual`](crate::combinator::contextual).
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

unsafe impl<T: Action<State = (), Heap = ()>, State, Heap> Action for Contextual<T, State, Heap> {
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
      state: &mut (),
      heap: &mut (),
    })
  }
}

/// Generate contextual combinators.
/// # Examples
/// ```
/// use whitehole::combinator::contextual;
/// # #[derive(Debug)]
/// # pub struct MyState;
/// # pub struct MyHeap;
///
/// // Generate contextual combinators with `MyState` and `MyHeap` as the state and heap types.
/// contextual!(MyState, MyHeap);
///
/// # fn main() {
/// // Use string combinators
/// let _ = take(1);
///
/// // Use byte combinators
/// let _ = bytes::take(1);
/// # }
/// ```
#[macro_export]
macro_rules! contextual {
  ($state:ty, $heap:ty) => {
    #[allow(dead_code)]
    mod _impl_contextual_combinators {
      #[allow(unused_imports)]
      use super::*;
      use std::{cell::OnceCell, rc::Rc};
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

      /// Contextual version of [`recur`](whitehole::combinator::recur).
      pub fn recur<Value>() -> (
        impl Fn() -> Combinator<$crate::combinator::Recur<$state, $heap, Value>>,
        $crate::combinator::RecurSetter<$state, $heap, Value>,
      ) {
        let inner = Rc::new(OnceCell::new());
        let setter = $crate::combinator::RecurSetter::new(inner.clone());
        let getter = move || Combinator::new($crate::combinator::Recur::new(inner.clone()));
        (getter, setter)
      }

      /// Contextual version of [`recur_unchecked`](whitehole::combinator::recur_unchecked).
      pub unsafe fn recur_unchecked<Value>() -> (
        impl Fn() -> Combinator<$crate::combinator::RecurUnchecked<$state, $heap, Value>>,
        $crate::combinator::RecurSetter<$state, $heap, Value>,
      ) {
        let inner = Rc::new(OnceCell::new());
        let setter = $crate::combinator::RecurSetter::new(inner.clone());
        let getter =
          move || Combinator::new($crate::combinator::RecurUnchecked::new(inner.clone()));
        (getter, setter)
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

        /// Contextual version of [`bytes::recur`](whitehole::combinator::bytes::recur).
        pub fn recur<Value>() -> (
          impl Fn() -> Combinator<$crate::combinator::bytes::Recur<$state, $heap, Value>>,
          $crate::combinator::bytes::RecurSetter<$state, $heap, Value>,
        ) {
          let inner = Rc::new(OnceCell::new());
          let setter = $crate::combinator::bytes::RecurSetter::new(inner.clone());
          let getter =
            move || Combinator::new($crate::combinator::bytes::Recur::new(inner.clone()));
          (getter, setter)
        }

        /// Contextual version of [`bytes::recur_unchecked`](whitehole::combinator::bytes::recur_unchecked).
        pub unsafe fn recur_unchecked<Value>() -> (
          impl Fn() -> Combinator<$crate::combinator::bytes::RecurUnchecked<$state, $heap, Value>>,
          $crate::combinator::bytes::RecurSetter<$state, $heap, Value>,
        ) {
          let inner = Rc::new(OnceCell::new());
          let setter = $crate::combinator::bytes::RecurSetter::new(inner.clone());
          let getter = move || {
            Combinator::new($crate::combinator::bytes::RecurUnchecked::new(
              inner.clone(),
            ))
          };
          (getter, setter)
        }
      }
    }
    pub use _impl_contextual_combinators::*;
  };
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_contextual() {
    contextual!(i32, i32);

    fn helper<Text: ?Sized>(_: impl Action<Text = Text, State = i32, Heap = i32>) {}

    helper(eat('a'));
    helper(take(1));
    helper(next(|_| true));
    helper(till('a'));
    helper(wrap(|input| input.instant.accept(0)));
    helper(unsafe { wrap_unchecked(|input| input.instant.accept(0)) });
    helper(recur::<()>().0());
    helper(unsafe { recur_unchecked::<()>() }.0());
    helper(bytes::eat(b'a'));
    helper(bytes::take(1));
    helper(bytes::next(|_| true));
    helper(bytes::till(b'a'));
    helper(bytes::wrap(|input| input.instant.accept(0)));
    helper(unsafe { bytes::wrap_unchecked(|input| input.instant.accept(0)) });
    helper(bytes::recur::<()>().0());
    helper(unsafe { bytes::recur_unchecked::<()>() }.0());

    // debug
    let action = take(1);
    let _ = format!("{:?}", action);
    // copy & clone
    let _c = action;
    let _c = action.clone();
  }
}
