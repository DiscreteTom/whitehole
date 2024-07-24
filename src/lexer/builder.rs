mod append;
mod define;
mod ignore;

use super::{action::Action, stateless::StatelessLexer, Lexer};
use crate::utils::OneOrMore;
use std::rc::Rc;

/// To create this, see [`Self::new`], [`Self::stateful`],
/// [`Self::with_error`] and [`Self::stateful_with_error`].
pub struct LexerBuilder<Kind: 'static, ActionState = (), ErrorType = ()> {
  actions: Vec<Action<Kind, ActionState, ErrorType>>,
}

impl<Kind, ActionState, ErrorType> Default for LexerBuilder<Kind, ActionState, ErrorType> {
  fn default() -> Self {
    Self {
      actions: Vec::new(),
    }
  }
}

impl<Kind> LexerBuilder<Kind> {
  /// Create a new lexer builder, set `ActionState` and `ErrorType` to `()`.
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
  pub fn new() -> Self {
    Self::default()
  }

  /// Create a new lexer builder with the provided `ActionState`,
  /// set `ErrorType` to `()`.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{LexerBuilder, action::exact};
  /// # struct MyState;
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
  pub fn stateful<ActionState>() -> LexerBuilder<Kind, ActionState> {
    LexerBuilder::default()
  }
}

impl<Kind, ErrorType> LexerBuilder<Kind, (), ErrorType> {
  /// Create a new lexer builder, set `ActionState` to `()`,
  /// infer `ErrorType` from the provided actions.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{LexerBuilder, action::exact};
  /// # #[derive(Clone)]
  /// # struct MyError;
  /// # let mut builder =
  /// LexerBuilder::with_error();
  /// # builder.append_with(exact("a"), |a| a.error(MyError));
  /// // equals to
  /// # let mut builder =
  /// LexerBuilder::<_, (), _>::default();
  /// # builder.append_with(exact("a"), |a| a.error(MyError));
  /// ```
  pub fn with_error() -> Self {
    Self::default()
  }

  /// Create a new lexer builder with the provided `ActionState`,
  /// infer `ErrorType` from the provided actions.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{LexerBuilder, action::exact};
  /// # struct MyState;
  /// # #[derive(Clone)]
  /// # struct MyError;
  /// # let mut builder =
  /// LexerBuilder::stateful_with_error::<MyState>();
  /// # builder.append_with(exact("a"), |a| a.error(MyError));
  /// // equals to
  /// # let mut builder =
  /// LexerBuilder::<_, MyState, _>::default();
  /// # builder.append_with(exact("a"), |a| a.error(MyError));
  /// ```
  pub fn stateful_with_error<ActionState>() -> LexerBuilder<Kind, ActionState, ErrorType> {
    LexerBuilder::default()
  }
}

impl<Kind, ActionState, ErrorType> LexerBuilder<Kind, ActionState, ErrorType> {
  // TODO: move into `generate`?
  pub fn build_stateless(self) -> StatelessLexer<Kind, ActionState, ErrorType> {
    // TODO: warning if action has no head matcher
    // wrap actions with Rc, make them immutable and clone-able
    StatelessLexer::new(self.actions.into_iter().map(Rc::new).collect())
  }

  pub fn build_with<'text>(
    self,
    action_state: ActionState,
    text: &'text str,
  ) -> Lexer<'text, Kind, ActionState, ErrorType> {
    Lexer::new(Rc::new(self.build_stateless()), action_state, text)
  }

  pub fn build<'text>(self, text: &'text str) -> Lexer<'text, Kind, ActionState, ErrorType>
  where
    ActionState: Default,
  {
    self.build_with(ActionState::default(), text)
  }

  fn map_actions<OldKind: 'static, NewKind>(
    actions: impl Into<OneOrMore<Action<OldKind, ActionState, ErrorType>>>,
    f: impl Fn(Action<OldKind, ActionState, ErrorType>) -> Action<NewKind, ActionState, ErrorType>,
  ) -> Vec<Action<NewKind, ActionState, ErrorType>> {
    actions.into().0.into_iter().map(f).collect()
  }
}
