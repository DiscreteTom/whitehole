#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output<Kind> {
  pub kind: Kind,
  /// How many bytes are digested by this action.
  /// # Caveats
  /// `0` is allowed, but be careful with infinite loops.
  ///
  /// This value should be smaller than or equal to the length of
  /// [`Input::rest`](crate::combinator::input::Input::rest).
  pub digested: usize,
}
