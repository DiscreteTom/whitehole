use crate::lexer::action::Action;

impl<Kind, State, Heap> Action<Kind, State, Heap> {
  /// Set [`Action::literal`].
  /// The provided parameter will NOT be checked, you have to make sure it's logically correct.
  ///
  /// This should only be used if you need extremely high performance.
  /// Otherwise you should use [`utils::exact`](crate::lexer::action::utils::exact)
  /// or [`utils::word`](crate::lexer::action::utils::word) to set [`Action::literal`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, simple}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// // in a valid json we know if a token starts with 't' then it must be "true"
  /// builder.define(A, simple(|_| 4).unchecked_head_in(['t']).unchecked_literal("true"));
  /// # }
  /// ```
  #[inline]
  pub fn unchecked_literal(mut self, s: impl Into<String>) -> Self {
    self.literal = Some(s.into());
    self
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::simple;

  #[test]
  fn action_literal() {
    let action: Action<_, ()> = simple(|_| 4);
    assert_eq!(action.literal(), &None);

    let action = action.unchecked_literal("abc");
    assert_eq!(action.literal(), &Some("abc".to_string()));
  }
}
