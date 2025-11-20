#[macro_export]
macro_rules! new_recur {
    ($(let $name:ident = $body:expr;)+) => {
      $(
        #[inline]
        const fn $name() -> $crate::combinator::Combinator<
          $crate::combinator::Contextual<$crate::combinator::WrapUnchecked<fn($crate::action::Input<&$crate::instant::Instant<&str>, &mut (), &mut ()>) -> Option<$crate::action::Output<()>>>, (), ()>,
        > {
          #[inline]
          fn $name(input: $crate::action::Input<&$crate::instant::Instant<&str>, &mut (), &mut ()>) -> Option<$crate::action::Output<()>> {
            ($body).exec(input)
          }
          unsafe { $crate::combinator::wrap_unchecked($name) }
        }
      )+
    };
}
