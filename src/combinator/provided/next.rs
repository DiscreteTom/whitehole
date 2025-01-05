use crate::{
  action::{Action, Input, Output},
  combinator::{create_closure_combinator, Combinator},
};

create_closure_combinator!(Next, "See [`next`].");

unsafe impl<State, Heap, F: Fn(char) -> bool> Action<State, Heap> for Next<F> {
  type Value = ();

  #[inline]
  fn exec(&self, input: Input<&mut State, &mut Heap>) -> Option<Output<Self::Value>> {
    let next = input.next();
    if !(self.inner)(next) {
      return None;
    }
    Some(unsafe { input.digest_unchecked(next.len_utf8()) })
  }
}

/// Returns a combinator to match
/// [`Input::next`](crate::action::Input::next) by the condition.
/// The combinator will reject if not matched.
/// # Examples
/// ```
/// # use whitehole::{combinator::{next, Combinator}, action::Action};
/// # fn t(_: Combinator<impl Action>) {}
/// // match one ascii digit
/// # t(
/// next(|c| c.is_ascii_digit())
/// # );
/// // match one or more ascii digit
/// # t(
/// next(|c| c.is_ascii_digit()) * (1..)
/// # );
/// ```
#[inline]
pub const fn next<F: Fn(char) -> bool>(condition: F) -> Combinator<Next<F>> {
  Combinator::new(Next::new(condition))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    action::{Action, Input},
    instant::Instant,
  };

  #[test]
  fn combinator_next() {
    // normal
    assert_eq!(
      next(|c| c.is_ascii_digit())
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(1)
    );
    // reject
    assert!(next(|c| c.is_ascii_alphabetic())
      .exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap())
      .is_none());

    // ensure the combinator is copyable and clone-able
    let c = next(|c| c.is_ascii_digit());
    let _ = c;
    let _ = c.clone();

    // ensure the combinator is debuggable
    assert_eq!(format!("{:?}", c), "Combinator { action: Next }");
  }

  #[test]
  fn one_or_more_next() {
    // normal
    assert_eq!(
      (next(|c| c.is_ascii_digit()) * (1..))
        .exec(Input::new(Instant::new("123"), &mut (), &mut ()).unwrap())
        .map(|output| output.digested),
      Some(3)
    );
    // reject
    assert!(next(|c| c.is_ascii_digit())
      .exec(Input::new(Instant::new("abc"), &mut (), &mut ()).unwrap())
      .is_none());
  }
}
