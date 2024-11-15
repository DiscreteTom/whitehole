//! Decorators that modify the acceptance of a combinator.

// use super::AcceptedOutputContext;
use crate::{
  combinator::{Input, Output, Parse},
  impl_combinator,
};

/// See [`Combinator::prevent`](crate::combinator::Combinator::prevent).
#[derive(Debug, Clone)]
pub struct Prevent<C, F> {
  c: C,
  f: F,
}

impl<C, F> Prevent<C, F> {
  #[inline]
  pub fn new(c: C, f: F) -> Self {
    Self { c, f }
  }
}

impl_combinator!(Prevent<C, F>, C, F);

impl<State, Heap, C: Parse<State, Heap>, F: Fn(&mut Input<&mut State, &mut Heap>) -> bool>
  Parse<State, Heap> for Prevent<C, F>
{
  type Kind = C::Kind;

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, Self::Kind>> {
    if (self.f)(input) {
      None
    } else {
      self.c.parse(input)
    }
  }
}

/// See [`Combinator::optional`](crate::combinator::Combinator::optional).
#[derive(Debug, Clone, Copy)]
pub struct Optional<C> {
  c: C,
}

impl<C> Optional<C> {
  #[inline]
  pub fn new(c: C) -> Self {
    Self { c }
  }
}

impl_combinator!(Optional<C>, C);

impl<State, Heap, C: Parse<State, Heap, Kind: Default>> Parse<State, Heap> for Optional<C> {
  type Kind = C::Kind;

  #[inline]
  fn parse<'text>(
    &self,
    input: &mut Input<'text, &mut State, &mut Heap>,
  ) -> Option<Output<'text, Self::Kind>> {
    Some(self.c.parse(input).unwrap_or_else(|| Output {
      kind: Default::default(),
      rest: input.rest(),
    }))
  }
}

// impl<'a, Kind: 'a, State: 'a, Heap: 'a> Combinator<'a, Kind, State, Heap> {
// /// Check the [`Input`] before the combinator is executed.
// /// Reject if the `condition` returns `true`.
// /// # Examples
// /// ```
// /// # use whitehole::combinator::Combinator;
// /// # fn t(combinator: Combinator<(), (), ()>) {
// /// combinator.prevent(|input| input.state.reject)
// /// # ;}
// /// ```
// pub fn prevent(self, condition: impl Fn(&mut Input<&mut State, &mut Heap>) -> bool + 'a) -> Self {
//   Combinator::boxed(move |input| {
//     if condition(input) {
//       None
//     } else {
//       self.parse(input)
//     }
//   })
// }

// /// Reject the combinator after execution if the `condition` returns `true`.
// /// # Examples
// /// ```
// /// # use whitehole::combinator::Combinator;
// /// # fn t(combinator: Combinator<(), (), ()>) {
// /// combinator.reject(|ctx| ctx.content() != "123")
// /// # ;}
// /// ```
// pub fn reject(
//   self,
//   condition: impl for<'text> Fn(
//       AcceptedOutputContext<&mut Input<'text, &mut State, &mut Heap>, &Output<'text, Kind>>,
//     ) -> bool
//     + 'a,
// ) -> Self {
//   Combinator::boxed(move |input| {
//     self.parse(input).and_then(|output| {
//       if condition(AcceptedOutputContext {
//         input,
//         output: &output,
//       }) {
//         None
//       } else {
//         output.into()
//       }
//     })
//   })
// }

// /// If the combinator is rejected, accept it with the default kind and zero digested.
// /// # Caveats
// /// This requires the `Kind` to implement [`Default`],
// /// thus usually used before setting a custom kind.
// /// ```
// /// # use whitehole::combinator::Combinator;
// /// # #[derive(Clone)]
// /// # enum MyKind { A }
// /// # fn t(combinator: Combinator<(), (), ()>) {
// /// // bind a kind after calling `optional`
// /// combinator.optional().bind(MyKind::A)
// /// // instead of
// /// // combinator.bind(MyKind::A).optional()
// /// # ;}
// /// ```
// /// Or you can wrap `Kind` with [`Option`]:
// /// ```
// /// # use whitehole::combinator::Combinator;
// /// # #[derive(Clone)]
// /// # enum MyKind { A }
// /// # fn t(combinator: Combinator<(), (), ()>) {
// /// combinator.bind(Some(MyKind::A)).optional()
// /// # ;}
// /// ```
// /// # Examples
// /// ```
// /// # use whitehole::combinator::Combinator;
// /// # fn t(combinator: Combinator<(), (), ()>) {
// /// combinator.optional()
// /// # ;}
// /// ```
// pub fn optional(self) -> Self
// where
//   Kind: Default,
// {
//   Combinator::boxed(move |input| {
//     Some(self.parse(input).unwrap_or_else(|| Output {
//       kind: Default::default(),
//       rest: input.rest(),
//     }))
//   })
// }

// /// Reject the combinator after execution if the next char is alphanumeric or `_`.
// /// See [`char::is_alphanumeric`].
// /// # Examples
// /// ```
// /// # use whitehole::combinator::Combinator;
// /// # fn t(combinator: Combinator<(), (), ()>) {
// /// combinator.boundary()
// /// # ;}
// /// ```
// pub fn boundary(self) -> Self {
//   self.reject(|ctx| {
//     ctx
//       .output
//       .rest
//       .chars()
//       .next()
//       .map_or(false, |c| c.is_alphanumeric() || c == '_')
//   })
// }
// }

// #[cfg(test)]
// mod tests {
//   use super::*;

//   fn accepter() -> Combinator<'static, (), bool, ()> {
//     Combinator::boxed(|input| {
//       *input.state = true;
//       Some(Output {
//         kind: (),
//         rest: &input.rest()[1..],
//       })
//     })
//   }

//   fn rejecter() -> Combinator<'static, (), bool, ()> {
//     Combinator::boxed(|input| {
//       *input.state = true;
//       None
//     })
//   }

//   #[test]
//   fn combinator_prevent() {
//     let mut executed = false;
//     assert!(accepter()
//       .prevent(|_| true)
//       .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap())
//       .is_none());
//     assert!(!executed);

//     let mut executed = false;
//     assert!(accepter()
//       .prevent(|_| false)
//       .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap())
//       .is_some());
//     assert!(executed);
//   }

//   #[test]
//   fn combinator_reject() {
//     let mut executed = false;
//     assert_eq!(
//       accepter()
//         .reject(|_| false)
//         .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap()),
//       Some(Output {
//         kind: (),
//         rest: "23"
//       })
//     );
//     assert!(executed);

//     let mut executed = false;
//     assert_eq!(
//       accepter()
//         .reject(|_| true)
//         .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap()),
//       None
//     );
//     assert!(executed);
//   }

//   #[test]
//   fn combinator_optional() {
//     let mut executed = false;
//     assert_eq!(
//       accepter()
//         .optional()
//         .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap()),
//       Some(Output {
//         kind: (),
//         rest: "23"
//       })
//     );
//     assert!(executed);

//     let mut executed = false;
//     assert_eq!(
//       rejecter()
//         .optional()
//         .parse(&mut Input::new("123", 0, &mut executed, &mut ()).unwrap()),
//       Some(Output {
//         kind: (),
//         rest: "123"
//       })
//     );
//     assert!(executed);
//   }

//   #[test]
//   fn combinator_boundary() {
//     let mut executed = false;
//     assert_eq!(
//       accepter()
//         .boundary()
//         .parse(&mut Input::new("1", 0, &mut executed, &mut ()).unwrap()),
//       Some(Output { kind: (), rest: "" })
//     );
//     assert!(executed);

//     let mut executed = false;
//     assert_eq!(
//       accepter()
//         .boundary()
//         .parse(&mut Input::new("12", 0, &mut executed, &mut ()).unwrap()),
//       None
//     );
//     assert!(executed);

//     let mut executed = false;
//     assert_eq!(
//       accepter()
//         .boundary()
//         .parse(&mut Input::new("1a", 0, &mut executed, &mut ()).unwrap()),
//       None
//     );
//     assert!(executed);

//     let mut executed = false;
//     assert_eq!(
//       accepter()
//         .boundary()
//         .parse(&mut Input::new("1_", 0, &mut executed, &mut ()).unwrap()),
//       None
//     );
//     assert!(executed);
//   }
// }
