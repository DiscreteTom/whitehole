//! ## Design
//!
//! For a better engineering experience, the lexer is designed to be modular
//! and consists of many [`Action`]s. Each action is a small piece of logic
//! which will digest some bytes from the rest of the text input, and optionally yield a token.
//! By doing so, users can easily compose their own lexer by re-using existing actions.
//! Users can also share their actions with others by publishing them as a library,
//! or build higher-level libraries to generate actions.
//!
//! ## For Developers
//!
//! Here is the recommended order of reading the source code:
//!
//! - [`self::input`]
//! - [`self::output`]
//! - [`self`]
//! - [`self::decorator`]
//! - [`self::simple`]
//! - [`self::utils`]

mod decorator;
mod input;
mod output;
mod simple;
mod utils;

pub use decorator::*;
pub use input::*;
pub use output::*;
pub use simple::*;
pub use utils::*;

use super::token::{TokenKindId, TokenKindIdBinding};
use std::{
  collections::HashSet,
  fmt::{self, Debug},
  rc::Rc,
};

/// See [`Action::head`].
#[derive(PartialEq, Debug, Clone)]
pub enum HeadMatcher {
  OneOf(HashSet<char>),
  Not(HashSet<char>),
  /// Match any characters that are not known in
  /// [`OneOf`](HeadMatcher::OneOf) or [`Not`](HeadMatcher::Not).
  Unknown,
}

#[derive(Debug)]
pub struct ActionBase<Kind, Exec> {
  // TODO: better name. e.g. digester?
  exec: Exec,

  /// See [`Self::kind`].
  kind: TokenKindId<Kind>,
  /// See [`Self::literal`].
  literal: Option<String>,
  /// See [`Self::head`].
  head: Option<HeadMatcher>,
  /// See [`Self::muted`].
  muted: bool,
}

// getters
impl<Kind, Exec> ActionBase<Kind, Exec> {
  /// This is used to accelerate expectational lexing if an expected kind is provided,
  /// see [`Expectation::kind`](crate::lexer::expectation::Expectation::kind) and
  /// the [`stateless`](crate::lexer::stateless) module for more details.
  ///
  /// Every action must have this field set by [`Self::bind`],
  /// [`Self::bind_default`] or [`Self::select`].
  /// These methods will ensure the integrity between [`Self::kind`] and [`ActionOutput::binding`].
  #[inline]
  pub const fn kind(&self) -> TokenKindId<Kind> {
    self.kind
  }

  /// This is used to accelerate expectational lexing if an expected literal is provided,
  /// see [`Expectation::literal`](crate::lexer::expectation::Expectation::literal) and
  /// the [`stateless`](crate::lexer::stateless) module for more details.
  /// If set, tokens' text content generated by this action must equals to this value
  /// (but this won't be checked during the runtime).
  ///
  /// This field is optional and should only be set via the [`exact`] util family and the [`word`] util family.
  /// If you are very sure about the literal, you could set this field manually by [`Self::unchecked_literal`].
  #[inline]
  pub const fn literal(&self) -> &Option<String> {
    &self.literal
  }

  /// This is used to accelerate lexing by the first character
  /// of the rest of the input.
  /// See the [`stateless`](crate::lexer::stateless) module for more details.
  /// This is optional but highly recommended.
  /// Some [`utils`] already set this field safely (e.g. [`exact`] and [`word`])
  /// and you should use them as much as possible.
  ///
  /// If you want to set this field manually,
  /// this could be set by [`Self::unchecked_head_in`], [`Self::unchecked_head_in_range`],
  /// [`Self::unchecked_head_not`] or [`Self::unchecked_head_unknown`].
  #[inline]
  pub const fn head(&self) -> &Option<HeadMatcher> {
    &self.head
  }

  /// Muted actions won't yield tokens and won't stop a lexing process from running,
  /// but the errors will still be collected by [`LexOptions::errors`](crate::lexer::options::LexOptions::errors).
  ///
  /// This field could be set via [`Self::mute`] or [`Self::unmute`].
  #[inline]
  pub const fn muted(&self) -> bool {
    self.muted
  }

  #[inline]
  pub const fn exec(&self) -> &Exec {
    &self.exec
  }
}

/// The [`Action::exec`].
/// This is a new-type for `Box<dyn Fn(...) -> ...>` and implements [`Debug`].
pub struct ActionExec<Kind, State, ErrorType> {
  pub(crate) raw: Box<
    dyn Fn(
      &mut ActionInput<&mut State>,
    ) -> Option<ActionOutput<TokenKindIdBinding<Kind>, Option<ErrorType>>>,
  >,
}

impl<Kind, State, ErrorType> ActionExec<Kind, State, ErrorType> {
  #[inline]
  pub(crate) fn new(
    raw: impl Fn(
        &mut ActionInput<&mut State>,
      ) -> Option<ActionOutput<TokenKindIdBinding<Kind>, Option<ErrorType>>>
      + 'static,
  ) -> Self {
    Self { raw: Box::new(raw) }
  }
}

impl<Kind, State, ErrorType> Debug for ActionExec<Kind, State, ErrorType> {
  #[inline]
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "ActionExec(...)")
  }
}

/// To create this, use [`simple`](simple::simple), [`simple_with_data`](simple::simple_with_data)
/// or [`utils`] (like [`regex`](utils::regex), [`exact`], [`word`]).
pub type Action<Kind, State = (), ErrorType = ()> =
  ActionBase<Kind, ActionExec<Kind, State, ErrorType>>;

/// Action's attributes without [`Action::exec`], wrapped in an [`Rc`].
pub(super) type RcActionProps<Kind> = Rc<ActionBase<Kind, ()>>;
/// [`Action::exec`] wrapped in an [`Rc`] to make it clone-able.
pub(super) type RcActionExec<Kind, State, ErrorType> = Rc<ActionExec<Kind, State, ErrorType>>;

impl<Kind, State, ErrorType> Action<Kind, State, ErrorType> {
  /// Break self into two parts and wrap them in [`Rc`].
  /// Return [`RcActionExec`] and [`RcActionProps`].
  #[inline]
  pub(super) fn into_rc(self) -> (RcActionExec<Kind, State, ErrorType>, RcActionProps<Kind>) {
    let props = Rc::new(ActionBase {
      kind: self.kind,
      literal: self.literal,
      head: self.head,
      muted: self.muted,
      exec: (),
    });
    (Rc::new(self.exec), props)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::token::SubTokenKind;
  use whitehole_macros::_token_kind;

  #[_token_kind]
  #[derive(Debug)]
  enum MyKind {
    A,
  }

  #[test]
  fn action_getters_default() {
    let action: Action<_> = Action {
      exec: ActionExec::new(|_| None),
      kind: A::kind_id(),
      head: None,
      muted: false,
      literal: None,
    };
    assert!(!action.muted());
    assert_eq!(action.kind(), A::kind_id());
    assert!(action.head().is_none());
    assert!(action.literal().is_none());
    assert!((action.exec().raw)(&mut ActionInput::new("1", 0, &mut ()).unwrap()).is_none())
  }

  #[test]
  fn action_getters() {
    let action: Action<_> = Action {
      exec: ActionExec::new(|_| None),
      kind: A::kind_id(),
      head: Some(HeadMatcher::OneOf(HashSet::from(['a']))),
      muted: true,
      literal: Some("123".into()),
    };
    assert!(action.muted());
    assert_eq!(action.kind(), A::kind_id());
    assert!(matches!(action.head(), Some(HeadMatcher::OneOf(set)) if set == &HashSet::from(['a'])));
    assert_eq!(action.literal(), &Some("123".into()));
    assert!((action.exec().raw)(&mut ActionInput::new("1", 0, &mut ()).unwrap()).is_none())
  }

  #[test]
  fn format_action() {
    assert_eq!(
      format!(
        "{:?}",
        Action::<_> {
          exec: ActionExec::new(|_| None),
          kind: A::kind_id(),
          head: Some(HeadMatcher::OneOf(HashSet::from(['a']))),
          muted: true,
          literal: Some("123".into()),
        }
      ),
      "ActionBase { exec: ActionExec(...), kind: TokenKindId<whitehole::lexer::action::tests::MyKind>(0), literal: Some(\"123\"), head: Some(OneOf({'a'})), muted: true }"
    );
  }
}
