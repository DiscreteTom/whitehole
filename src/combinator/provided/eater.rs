use crate::{
  action::{Action, Input, Output},
  combinator::{closure_combinator, Combinator},
};

closure_combinator!(Eater, "See [`eater`].");
closure_combinator!(EaterUnchecked, "See [`eater_unchecked`].");

macro_rules! impl_eater {
  ($name:ident, $output:expr) => {
    unsafe impl<State, Heap, F: Fn(Input<&mut State, &mut Heap>) -> usize> Action
      for $name<F, State, Heap>
    {
      type Value = ();
      type State = State;
      type Heap = Heap;

      #[inline]
      fn exec(
        &self,
        mut input: Input<&mut Self::State, &mut Self::Heap>,
      ) -> Option<Output<Self::Value>> {
        match (self.f)(input.reborrow()) {
          0 => None,
          digested => $output(&input, digested),
        }
      }
    }
  };
}

#[inline(always)]
fn build_output_unchecked<State, Heap>(
  input: &Input<&mut State, &mut Heap>,
  digested: usize,
) -> Option<Output<()>> {
  unsafe { input.digest_unchecked(digested) }.into()
}

impl_eater!(Eater, Input::digest);
impl_eater!(EaterUnchecked, build_output_unchecked);

/// Returns a combinator by the provided function that
/// eats [`Instant::rest`](crate::instant::Instant::rest) and returns the number of digested bytes (not chars).
/// The combinator will reject if the function returns `0`
/// or [`Output::digested`](crate::action::Output::digested) is invalid.
/// # Examples
/// ```
/// # use whitehole::{combinator::eater, C};
/// # fn t(_: C!()) {}
/// // accept all the rest characters
/// # t(
/// eater(|input| input.instant().rest().len())
/// # );
/// ```
#[inline]
pub const fn eater<State, Heap, F: Fn(Input<&mut State, &mut Heap>) -> usize>(
  f: F,
) -> Combinator<Eater<F, State, Heap>> {
  Combinator::new(Eater::new(f))
}

/// Returns a combinator by the provided function that
/// eats [`Instant::rest`](crate::instant::Instant::rest) and returns the number of digested bytes (not chars).
/// The combinator will reject if the function returns `0`.
/// # Safety
/// You should ensure that the [`Output::digested`](crate::action::Output::digested) is valid.
/// This will be checked using [`debug_assert!`].
/// For the checked version, see [`eater`].
/// # Examples
/// ```
/// # use whitehole::{combinator::eater_unchecked, C};
/// # fn t(_: C!()) {}
/// // accept all the rest characters
/// # t(
/// unsafe { eater_unchecked(|input| input.instant().rest().len()) }
/// # );
/// ```
#[inline]
pub const unsafe fn eater_unchecked<State, Heap, F: Fn(Input<&mut State, &mut Heap>) -> usize>(
  f: F,
) -> Combinator<EaterUnchecked<F, State, Heap>> {
  Combinator::new(EaterUnchecked::new(f))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{action::Action, instant::Instant};

  #[test]
  fn combinator_eater() {
    // normal
    assert_eq!(
      eater(|input| input.instant().rest().len())
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // overflow
    assert_eq!(
      eater(|input| input.instant().rest().len() + 1)
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      None
    );
    // invalid code point
    assert_eq!(
      eater(|_| 1)
        .exec(Input::new(Instant::new("好"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      None
    );
    // 0
    assert_eq!(
      eater(|_| 0)
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      None
    );

    // ensure the combinator is copyable and clone-able
    let c = eater::<(), (), _>(|_| 1);
    let _ = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(format!("{:?}", c), "Combinator { action: Eater }");
  }

  #[test]
  fn combinator_eater_unchecked() {
    // normal
    assert_eq!(
      unsafe { eater_unchecked(|input| input.instant().rest().len()) }
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // 0
    assert_eq!(
      unsafe { eater_unchecked(|_| 0) }
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      None
    );

    // ensure the combinator is copyable and clone-able
    let c = unsafe { eater_unchecked::<(), (), _>(|_| 1) };
    let _ = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(format!("{:?}", c), "Combinator { action: EaterUnchecked }");
  }

  #[test]
  #[should_panic]
  fn combinator_eater_unchecked_overflow() {
    unsafe { eater_unchecked(|input| input.instant().rest().len() + 1) }
      .exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap());
  }

  #[test]
  #[should_panic]
  fn combinator_eater_unchecked_invalid_code_point() {
    unsafe { eater_unchecked(|_| 1) }
      .exec(Input::new(Instant::new("好"), &mut (), &mut ()).unwrap());
  }
}
