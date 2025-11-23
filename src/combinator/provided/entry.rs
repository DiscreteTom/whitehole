use crate::{
  action::{Input, Output},
  combinator::{Contextual, WrapUnchecked},
  instant::Instant,
};

pub type Entry<Value = (), State = (), Heap = ()> = Contextual<
  WrapUnchecked<fn(Input<&Instant<&str>, &mut State, &mut Heap>) -> Option<Output<Value>>>,
  State,
  Heap,
>;

#[macro_export]
macro_rules! entry {
  ($(let $name:ident = || $body:expr;)+) => {
    $(
      #[inline]
      pub const fn $name() -> $crate::combinator::Combinator<$crate::combinator::Entry> {
        #[inline]
        fn inner(input: $crate::action::Input<&$crate::instant::Instant<&str>, &mut (), &mut ()>) -> Option<$crate::action::Output<()>> {
          ($body).exec(input)
        }
        unsafe { $crate::combinator::wrap_unchecked(inner) }
      }
    )+
  };
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::action::Action;
  use crate::combinator::eat;

  #[test]
  fn test_entry() {
    entry! {
      let value = || eat('a') | array();
      let array = || eat('[') + (value() * (..)).sep(',') + eat(']');
    }
  }
}
