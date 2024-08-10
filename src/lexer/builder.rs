mod append;
mod define;
mod ignore;

use super::{action::Action, stateless::StatelessLexer, Lexer};
use crate::utils::OneOrMore;
use std::rc::Rc;

/// To create this, see [`Self::new`], [`Self::stateful`],
/// [`Self::with_error`] and [`Self::stateful_with_error`].
pub struct LexerBuilder<Kind, State = (), ErrorType = ()> {
  actions: Vec<Action<Kind, State, ErrorType>>,
}

impl<Kind, State, ErrorType> Default for LexerBuilder<Kind, State, ErrorType> {
  fn default() -> Self {
    Self {
      actions: Vec::new(),
    }
  }
}

impl<Kind> LexerBuilder<Kind> {
  /// Create a new lexer builder, set `State` and `ErrorType` to `()`.
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

  /// Create a new lexer builder with the provided `State`,
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
  pub fn stateful<State>() -> LexerBuilder<Kind, State> {
    LexerBuilder::default()
  }
}

impl<Kind, ErrorType> LexerBuilder<Kind, (), ErrorType> {
  /// Create a new lexer builder, set `State` to `()`,
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

  /// Create a new lexer builder with the provided `State`,
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
  pub fn stateful_with_error<State>() -> LexerBuilder<Kind, State, ErrorType> {
    LexerBuilder::default()
  }
}

impl<Kind, State, ErrorType> LexerBuilder<Kind, State, ErrorType> {
  // TODO: move into `generate`?
  pub fn build_stateless(self) -> StatelessLexer<Kind, State, ErrorType> {
    // TODO: warning if action has no head matcher
    // wrap actions with Rc, make them immutable and clone-able
    StatelessLexer::new(self.actions)
  }

  pub fn build_with<'text>(
    self,
    state: State,
    text: &'text str,
  ) -> Lexer<'text, Kind, State, ErrorType> {
    Lexer::new(Rc::new(self.build_stateless()), state, text)
  }

  pub fn build<'text>(self, text: &'text str) -> Lexer<'text, Kind, State, ErrorType>
  where
    State: Default,
  {
    self.build_with(State::default(), text)
  }

  fn map_actions<OldKind, NewKind>(
    actions: impl Into<OneOrMore<Action<OldKind, State, ErrorType>>>,
    f: impl Fn(Action<OldKind, State, ErrorType>) -> Action<NewKind, State, ErrorType>,
  ) -> Vec<Action<NewKind, State, ErrorType>> {
    actions.into().0.into_iter().map(f).collect()
  }
}
