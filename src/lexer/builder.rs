mod action_list;
mod append;
mod define;
mod ignore;

pub use action_list::*;

use super::{action::Action, stateless::StatelessLexer, Lexer};
use std::rc::Rc;

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

impl<Kind, ActionState, ErrorType> LexerBuilder<Kind, ActionState, ErrorType> {
  /// Equals to [`Self::default`].
  pub fn new() -> Self {
    Self::default()
  }

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
    actions: impl Into<ActionList<Action<OldKind, ActionState, ErrorType>>>,
    f: impl Fn(Action<OldKind, ActionState, ErrorType>) -> Action<NewKind, ActionState, ErrorType>,
  ) -> Vec<Action<NewKind, ActionState, ErrorType>> {
    actions.into().0.into_iter().map(f).collect()
  }
}
