//! Decorators that modify the acceptance of a combinator.

use super::AcceptedOutputContext;
use crate::combinator::{wrap, Combinator, Input, Output, Parse};

impl<T> Combinator<T> {
  pub fn optional<State, Heap>(self) -> Combinator<impl Parse<State, Heap, Kind = T::Kind>>
  where
    T: Parse<State, Heap>,
    T::Kind: Default,
  {
    wrap(move |input| {
      Some(self.parse(input).unwrap_or_else(|| Output {
        kind: Default::default(),
        rest: input.rest(),
      }))
    })
  }
}

// /// See [`Combinator::prevent`](crate::combinator::Combinator::prevent).
// #[derive(Debug, Clone)]
// pub struct Prevent<C, F> {
//   c: C,
//   f: F,
// }

// impl<C, F> Prevent<C, F> {
//   #[inline]
//   pub fn new(c: C, f: F) -> Self {
//     Self { c, f }
//   }
// }

// impl<State, Heap, C: Parse<State, Heap>, F: Fn(&mut Input<&mut State, &mut Heap>) -> bool>
//   Parse<State, Heap> for Prevent<C, F>
// {
//   type Kind = C::Kind;

//   #[inline]
//   fn parse<'text>(
//     &self,
//     input: &mut Input<'text, &mut State, &mut Heap>,
//   ) -> Option<Output<'text, Self::Kind>> {
//     if (self.f)(input) {
//       None
//     } else {
//       self.c.parse(input)
//     }
//   }
// }

// /// See [`Combinator::optional`](crate::combinator::Combinator::optional).
// #[derive(Debug, Clone, Copy)]
// pub struct Optional<C> {
//   c: C,
// }

// impl<C> Optional<C> {
//   #[inline]
//   pub fn new(c: C) -> Self {
//     Self { c }
//   }
// }

// impl_combinator!(Optional<C>, C);

// impl<State, Heap, C: Parse<State, Heap, Kind: Default>> Parse<State, Heap> for Optional<C> {
//   type Kind = C::Kind;

//   #[inline]
//   fn parse<'text>(
//     &self,
//     input: &mut Input<'text, &mut State, &mut Heap>,
//   ) -> Option<Output<'text, Self::Kind>> {
//     Some(self.c.parse(input).unwrap_or_else(|| Output {
//       kind: Default::default(),
//       rest: input.rest(),
//     }))
//   }
// }

// /// See [`Combinator::reject`](crate::combinator::Combinator::reject).
// #[derive(Debug, Clone)]
// pub struct Reject<C, F> {
//   c: C,
//   f: F,
// }

// impl<C, F> Reject<C, F> {
//   #[inline]
//   pub fn new(c: C, f: F) -> Self {
//     Self { c, f }
//   }
// }

// impl_combinator!(Reject<C, F>, C, F);

// impl<
//     State,
//     Heap,
//     C: Parse<State, Heap>,
//     F: for<'text> Fn(
//       AcceptedOutputContext<&mut Input<'text, &mut State, &mut Heap>, &Output<'text, C::Kind>>,
//     ) -> bool,
//   > Parse<State, Heap> for Reject<C, F>
// {
//   type Kind = C::Kind;

//   #[inline]
//   fn parse<'text>(
//     &self,
//     input: &mut Input<'text, &mut State, &mut Heap>,
//   ) -> Option<Output<'text, Self::Kind>> {
//     self.c.parse(input).and_then(|output| {
//       if (self.f)(AcceptedOutputContext {
//         input,
//         output: &output,
//       }) {
//         None
//       } else {
//         output.into()
//       }
//     })
//   }
// }

// /// See [`Combinator::boundary`](crate::combinator::Combinator::boundary).
// #[derive(Debug, Clone, Copy)]
// pub struct Boundary<C> {
//   c: C,
// }

// impl<C> Boundary<C> {
//   #[inline]
//   pub fn new(c: C) -> Self {
//     Self { c }
//   }
// }

// impl_combinator!(Boundary<C>, C);

// impl<State, Heap, C: Parse<State, Heap>> Parse<State, Heap> for Boundary<C> {
//   type Kind = C::Kind;

//   #[inline]
//   fn parse<'text>(
//     &self,
//     input: &mut Input<'text, &mut State, &mut Heap>,
//   ) -> Option<Output<'text, Self::Kind>> {
//     let output = self.c.parse(input)?;
//     if output
//       .rest
//       .chars()
//       .next()
//       .map_or(false, |c| c.is_alphanumeric() || c == '_')
//     {
//       None
//     } else {
//       Some(output)
//     }
//   }
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
