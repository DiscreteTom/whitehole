mod append;
mod define;
mod ignore;

use super::{action::Action, lexer::IntoLexer, stateless::StatelessLexer, Lexer};
use crate::utils::OneOrMore;

/// To create this, see [`Self::new`] and [`Self::stateful`].
#[derive(Debug)]
pub struct LexerBuilder<Kind, State = ()> {
  actions: Vec<Action<Kind, State>>,
}

impl<Kind, State> Default for LexerBuilder<Kind, State> {
  #[inline]
  fn default() -> Self {
    Self {
      actions: Vec::new(),
    }
  }
}

impl<Kind> LexerBuilder<Kind> {
  /// Create a new lexer builder, set `State` to `()`,
  /// auto infer `Kind` from the provided actions.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{LexerBuilder, action::exact};
  /// # let mut builder =
  /// LexerBuilder::new();
  /// # builder.append(exact("a"));
  /// // equals to
  /// # let mut builder =
  /// LexerBuilder::<_>::default();
  /// # builder.append(exact("a"));
  /// # let mut builder =
  /// LexerBuilder::<_, (), ()>::default();
  /// # builder.append(exact("a"));
  /// ```
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  /// Create a new lexer builder with the provided `State`,
  /// auto infer `Kind` from the provided actions.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{LexerBuilder, action::exact};
  /// # struct MyState;
  /// # let mut builder: LexerBuilder<_, MyState> =
  /// LexerBuilder::stateful();
  /// # builder.append(exact("a"));
  /// # let mut builder =
  /// LexerBuilder::stateful::<MyState>();
  /// # builder.append(exact("a"));
  /// // equals to
  /// # let mut builder =
  /// LexerBuilder::<_, MyState>::default();
  /// # builder.append(exact("a"));
  /// # let mut builder =
  /// LexerBuilder::<_, MyState, ()>::default();
  /// # builder.append(exact("a"));
  /// ```
  #[inline]
  pub fn stateful<State>() -> LexerBuilder<Kind, State> {
    LexerBuilder::default()
  }
}

impl<Kind, State> LexerBuilder<Kind, State> {
  /// Get the actions appended to this instance.
  #[inline]
  pub fn actions(&self) -> &[Action<Kind, State>] {
    &self.actions
  }

  /// Check if all actions have the head matcher set. See [`Action::head`].
  ///
  /// Return [`Err`] with no-head-matcher action indexes.
  /// # Examples
  /// This should be used after all actions are appended, before build.
  /// ```
  /// # use whitehole::lexer::{LexerBuilder, action::exact};
  /// LexerBuilder::new()
  ///   .append(exact("a"))
  ///   .ensure_head_matcher()
  ///   .unwrap()
  ///   .build("a");
  /// ```
  pub fn ensure_head_matcher(self) -> Result<Self, (Vec<usize>, Self)> {
    let mut invalid = vec![];
    for (i, a) in self.actions.iter().enumerate() {
      if a.head().is_none() {
        invalid.push(i);
      }
    }
    if invalid.is_empty() {
      Ok(self)
    } else {
      Err((invalid, self))
    }
  }

  // TODO: add a module `generate` to speed up the build process? store action index & lookup tables.
  /// Consume self, build a [`StatelessLexer`].
  #[inline]
  pub fn build_stateless(self) -> StatelessLexer<Kind, State> {
    StatelessLexer::new(self.actions)
  }

  /// Alias of [`Self::into_lexer_with`].
  #[inline]
  pub fn build_with<'text>(self, state: State, text: &'text str) -> Lexer<'text, Kind, State> {
    self.into_lexer_with(state, text)
  }

  /// Alias of [`Self::into_lexer`].
  #[inline]
  pub fn build<'text>(self, text: &'text str) -> Lexer<'text, Kind, State>
  where
    State: Default,
  {
    self.into_lexer(text)
  }

  #[inline]
  fn map_actions<OldKind, NewKind>(
    actions: impl Into<OneOrMore<Action<OldKind, State>>>,
    f: impl Fn(Action<OldKind, State>) -> Action<NewKind, State>,
  ) -> Vec<Action<NewKind, State>> {
    actions.into().0.into_iter().map(f).collect()
  }
}

impl<Kind, State> IntoLexer<Kind, State> for LexerBuilder<Kind, State> {
  #[inline]
  fn into_lexer_with(self, state: State, text: &str) -> Lexer<Kind, State> {
    self.build_stateless().into_lexer_with(state, text)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::{
    action::{exact, regex},
    token::MockTokenKind,
  };

  #[test]
  fn test_lexer_builder_constructors() {
    // ensure the return type is correct
    let _: LexerBuilder<MockTokenKind<()>, i32> = LexerBuilder::default().append(exact("a"));
    let _: LexerBuilder<MockTokenKind<()>> = LexerBuilder::new().append(exact("a"));
    let _: LexerBuilder<MockTokenKind<()>, i32> = LexerBuilder::stateful().append(exact("a"));
  }

  #[test]
  fn test_lexer_builder_actions() {
    let builder = LexerBuilder::new().append(exact("a"));
    assert_eq!(builder.actions().len(), 1);
  }

  #[test]
  fn test_lexer_builder_ensure_head_matcher() {
    let (invalid, _) = LexerBuilder::new()
      .append([regex("a"), exact("a"), regex("a")])
      .ensure_head_matcher()
      .unwrap_err();
    assert_eq!(invalid, vec![0, 2]);

    LexerBuilder::new()
      .append(exact("a"))
      .ensure_head_matcher()
      .unwrap();
  }

  #[test]
  fn test_build() {
    let builder_factory = || LexerBuilder::stateful::<i32>().append(exact("a"));

    let lexer = builder_factory().build_with(1, "123");
    assert_eq!(lexer.state, 1);
    assert_eq!(lexer.instant().text(), "123");

    let lexer = builder_factory().build("123");
    assert_eq!(lexer.state, 0);
    assert_eq!(lexer.instant().text(), "123");
  }
}
