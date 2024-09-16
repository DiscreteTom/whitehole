use crate::kind::KindIdBinding;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionOutput<Kind> {
  /// The [`Token::binding`](crate::lexer::token::Token::binding)
  /// that is created by this action.
  pub binding: KindIdBinding<Kind>,
  /// How many bytes are digested by this action.
  /// # Caveats
  /// `0` is allowed, but be careful with infinite loops.
  ///
  /// This value should be smaller than or equal to the length of
  /// [`ActionInput::rest`](crate::lexer::action::input::ActionInput::rest).
  pub digested: usize,
}

impl<Kind> ActionOutput<Kind> {
  /// Convert [`Self::binding`] to another kind.
  #[inline]
  pub(super) fn map<NewKind>(
    self,
    f: impl FnOnce(Self) -> KindIdBinding<NewKind>,
  ) -> ActionOutput<NewKind> {
    ActionOutput {
      digested: self.digested,
      binding: f(self),
    }
  }
}
