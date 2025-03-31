use crate::{
  action::{Action, Input, Output},
  combinator::{provided::create_closure_combinator, Combinator, Contextual},
  digest::Digest,
  instant::Instant,
};

create_closure_combinator!(WrapUnchecked, "See [`wrap_unchecked`].");
create_closure_combinator!(Wrap, "See [`wrap`].");

macro_rules! impl_wrap {
  ($name:ident, $assert:ident, $text:ty) => {
    unsafe impl<
        State,
        Heap,
        Value,
        F: Fn(Input<&Instant<&$text>, &mut State, &mut Heap>) -> Option<Output<Value>>,
      > Action for Contextual<$name<F>, State, Heap>
    {
      type Text = $text;
      type State = State;
      type Heap = Heap;
      type Value = Value;

      #[inline]
      fn exec(
        &self,
        input: Input<&Instant<&Self::Text>, &mut Self::State, &mut Self::Heap>,
      ) -> Option<Output<Self::Value>> {
        let instant = input.instant;
        let output = (self.action.inner)(input);
        $assert!(output
          .as_ref()
          .map_or(true, |output| instant.rest().validate(output.digested)));
        output
      }
    }
  };
}
pub(super) use impl_wrap;

impl_wrap!(WrapUnchecked, debug_assert, str);
impl_wrap!(Wrap, assert, str);

// TODO: for non-contextual version, can we just return Combinator<Wrap>?

/// Wrap a closure or function to create a [`Combinator`].
/// # Safety
/// The returned [`Output`] should satisfy the requirement of [`Output::digested`].
/// This will be checked using [`debug_assert!`].
/// For the checked version, see [`wrap`].
/// # Examples
/// ```
/// # use whitehole::combinator::{wrap_unchecked, Combinator};
/// # use whitehole::action::{Output, Action};
/// # fn t() -> Combinator<impl Action> {
/// // accept with 0 bytes digested
/// unsafe { wrap_unchecked(|input| input.instant.accept(0)) }
/// # }
/// ```
#[inline]
pub const unsafe fn wrap_unchecked<
  Value,
  F: Fn(Input<&Instant<&str>, &mut (), &mut ()>) -> Option<Output<Value>>,
>(
  f: F,
) -> Combinator<Contextual<WrapUnchecked<F>, (), ()>> {
  Combinator::new(Contextual::new(WrapUnchecked::new(f)))
}

/// Wrap a closure or function to create a [`Combinator`].
/// # Panics
/// The returned [`Output`] should satisfy the requirement of [`Output::digested`],
/// otherwise the combinator will panic when executed.
/// # Examples
/// ```
/// # use whitehole::combinator::{wrap, Combinator};
/// # use whitehole::action::{Output, Action};
/// # fn t() -> Combinator<impl Action> {
/// // accept with 0 bytes digested
/// wrap(|input| input.instant.accept(0))
/// # }
/// ```
#[inline]
pub const fn wrap<
  Value,
  F: Fn(Input<&Instant<&str>, &mut (), &mut ()>) -> Option<Output<Value>>,
>(
  f: F,
) -> Combinator<Contextual<Wrap<F>, (), ()>> {
  Combinator::new(Contextual::new(Wrap::new(f)))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::instant::Instant;
  use std::{ops::RangeFrom, slice::SliceIndex};

  fn helper<Text: ?Sized + Digest>(
    action: impl Action<Text = Text, State = (), Heap = (), Value = ()>,
    input: &Text,
    digested: usize,
  ) where
    RangeFrom<usize>: SliceIndex<Text, Output = Text>,
  {
    assert_eq!(
      action
        .exec(Input {
          instant: &Instant::new(input),
          state: &mut (),
          heap: &mut ()
        })
        .unwrap()
        .digested,
      digested
    )
  }

  #[test]
  fn combinator_wrap_unchecked() {
    let c = unsafe { wrap_unchecked(|input| input.instant.accept(1)) };
    helper(c, "1", 1);

    // ensure the combinator is copyable and clone-able
    let _c = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(
      format!("{:?}", c),
      "Combinator { action: Contextual(WrapUnchecked) }"
    );
  }

  #[test]
  fn combinator_wrap_unchecked_fn() {
    fn action(input: Input<&Instant<&str>, &mut (), &mut ()>) -> Option<Output<()>> {
      input.instant.accept(1)
    }
    let c = unsafe { wrap_unchecked(action) };
    helper(c, "1", 1);

    // ensure the combinator is copyable and clone-able
    let _c = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(
      format!("{:?}", c),
      "Combinator { action: Contextual(WrapUnchecked) }"
    );
  }

  #[test]
  #[should_panic]
  fn combinator_wrap_unchecked_overflow() {
    helper(
      unsafe { wrap_unchecked(|input| input.instant.accept_unchecked(4).into()) },
      "1",
      0,
    );
  }

  #[test]
  #[should_panic]
  fn combinator_wrap_unchecked_invalid_code_point() {
    helper(
      unsafe { wrap_unchecked(|input| input.instant.accept_unchecked(1).into()) },
      "好",
      0,
    );
  }

  #[test]
  fn combinator_wrap() {
    let c = wrap(|input| input.instant.accept(1));
    helper(c, "1", 1);

    // ensure the combinator is copyable and clone-able
    let _c = c;
    let _c = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(
      format!("{:?}", c),
      "Combinator { action: Contextual(Wrap) }"
    );
  }

  #[test]
  fn combinator_wrap_fn() {
    fn action(input: Input<&Instant<&str>, &mut (), &mut ()>) -> Option<Output<()>> {
      input.instant.accept(1)
    }
    let c = wrap(action);
    helper(c, "1", 1);

    // ensure the combinator is copyable and clone-able
    let _c = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(
      format!("{:?}", c),
      "Combinator { action: Contextual(Wrap) }"
    );
  }

  #[test]
  #[should_panic]
  fn combinator_wrap_overflow() {
    helper(
      wrap(|input| unsafe { input.instant.accept_unchecked(4) }.into()),
      "1",
      0,
    );
  }

  #[test]
  #[should_panic]
  fn combinator_wrap_invalid_code_point() {
    helper(
      wrap(|input| unsafe { input.instant.accept_unchecked(1) }.into()),
      "好",
      0,
    );
  }
}
