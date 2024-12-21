/// A helper trait to accumulate values when performing `*` on [`Combinator`]s.
///
/// Built-in implementations are provided for `()`.
/// # Examples
/// ## Inline Fold
/// For simple cases, you can accumulate the values inline, without using this trait.
/// ```
/// # use whitehole::{combinator::next, action::{Input, Action}};
/// let combinator =
///   // accept one ascii digit at a time
///   next(|c| c.is_ascii_digit())
///     // convert the char to a number
///     .select(|ctx| ctx.input.next() as usize - '0' as usize)
///     // repeat for 1 or more times, init accumulator with 0, and fold values
///     * (1.., || 0 as usize, |value, acc| acc * 10 + value);
///
/// // parse "123" to 123
/// assert_eq!(
///   combinator.exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()).unwrap().value,
///   123
/// )
/// ```
/// ## Fold with Custom Type
/// If you want to re-use the folder logic, you can implement this trait for a custom type.
/// ```
/// # use whitehole::{combinator::{ops::mul::Fold, next}, action::{Input, Action}};
/// // since you can't implement `Fold` for `usize` directly,
/// // wrap it in a new-type
/// struct Usize(usize);
///
/// impl Fold for Usize {
///   type Output = usize;
///
///   fn fold(self, acc: Self::Output) -> Self::Output {
///     acc * 10 + self.0
///   }
/// }
///
/// let combinator =
///   // accept one ascii digit at a time
///   next(|c| c.is_ascii_digit())
///     // convert the char to a number, wrapped in `Usize`
///     .select(|ctx| Usize(ctx.input.next() as usize - '0' as usize))
///     // repeat for 1 or more times, fold `Usize` to `usize`
///     * (1..);
///     // equals to: `* (1.., Usize::Output::default, Usize::fold)`
///
/// // parse "123" to 123
/// assert_eq!(
///   combinator.exec(&mut Input::new("123", 0, &mut (), &mut ()).unwrap()).unwrap().value,
///   123
/// )
/// ```
pub trait Fold {
  /// The accumulator type.
  type Output: Default;

  /// Fold self with the accumulator.
  fn fold(self, acc: Self::Output) -> Self::Output;
}

impl Fold for () {
  type Output = ();
  #[inline]
  fn fold(self, _: Self::Output) -> Self::Output {}
}
