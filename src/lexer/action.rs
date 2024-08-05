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
use std::{collections::HashSet, rc::Rc};

/// See [`Action::head`].
#[derive(PartialEq, Debug, Clone)]
pub enum HeadMatcher {
  OneOf(HashSet<char>),
  Not(HashSet<char>),
  /// Match any characters that are not known in
  /// [`OneOf`](HeadMatcher::OneOf) or [`Not`](HeadMatcher::Not).
  Unknown,
}

/// This is the common part of [`Action`], [`ImmutableAction`] and [`MutableAction`].
/// See [`GeneralAction`] for more details.
pub struct ActionBase<Kind: 'static, Exec> {
  /// See [`Self::kind`].
  kind: &'static TokenKindId<Kind>,
  /// See [`Self::literal`].
  literal: Option<String>,
  /// See [`Self::head`].
  head: Option<HeadMatcher>,
  /// See [`Self::muted`].
  muted: bool,

  exec: Exec,
}

// getters
impl<Kind, Exec> ActionBase<Kind, Exec> {
  /// This is used to accelerate expectational lexing if an expected kind is provided,
  /// see [`Expectation::kind`](crate::lexer::expectation::Expectation::kind).
  ///
  /// Every action must have this field set by [`Self::bind`],
  /// [`Self::bind_default`] or [`Self::select`].
  /// These method will ensure the integrity between [`Self::kind`] and [`ActionOutput::binding`].
  #[inline]
  pub const fn kind(&self) -> &'static TokenKindId<Kind> {
    &self.kind
  }

  /// This is used to accelerate expectational lexing if an expected literal is provided,
  /// see [`Expectation::literal`](crate::lexer::expectation::Expectation::literal).
  /// If set, tokens' text content generated by this action must equals to this value
  /// (but this won't be checked during the runtime).
  ///
  /// This field is optional and can only be set via [`exact`] and [`word`].
  #[inline]
  pub const fn literal(&self) -> &Option<String> {
    &self.literal
  }

  /// This is used to accelerate lexing by the first character
  /// of the rest of the input. This is optional but highly recommended.
  /// Some [`utils`] already set this field safely (e.g. [`exact`] and [`word`]) and you should use them as much as possible.
  ///
  /// If you want to set this field manually,
  /// this could be set by [`Self::unchecked_head_in`], [`Self::unchecked_head_in_range`],
  /// [`Self::unchecked_head_not`] or [`Self::unchecked_head_unknown`].
  #[inline]
  pub const fn head(&self) -> &Option<HeadMatcher> {
    &self.head
  }

  /// Muted actions won't yield tokens and won't stop a lexing process from running,
  /// but the errors will still be collected by [`LexOutput::errors`](crate::lexer::output::LexOutput::errors).
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

/// [`Action::exec`] that won't mutate the action state.
type ImmutableActionExec<Kind, State, ErrorType> = Box<
  dyn Fn(&ActionInput<&State>) -> Option<ActionOutput<TokenKindIdBinding<Kind>, Option<ErrorType>>>,
>;

/// [`Action::exec`] that will mutate the action state.
type MutableActionExec<Kind, State, ErrorType> = Box<
  dyn Fn(
    &mut ActionInput<&mut State>,
  ) -> Option<ActionOutput<TokenKindIdBinding<Kind>, Option<ErrorType>>>,
>;

/// See [`Action::exec`].
pub enum ActionExec<Kind: 'static, State, ErrorType> {
  Immutable(ImmutableActionExec<Kind, State, ErrorType>),
  Mutable(MutableActionExec<Kind, State, ErrorType>),
}

/// To create this, use [`simple`](simple::simple), [`simple_with_data`](simple::simple_with_data)
/// or [`utils`] (like [`regex`](utils::regex), [`exact`], [`word`]).
pub type Action<Kind, State = (), ErrorType = ()> =
  ActionBase<Kind, ActionExec<Kind, State, ErrorType>>;

pub(super) type ImmutableAction<Kind, State, ErrorType> =
  ActionBase<Kind, ImmutableActionExec<Kind, State, ErrorType>>;
pub(super) type MutableAction<Kind, State, ErrorType> =
  ActionBase<Kind, MutableActionExec<Kind, State, ErrorType>>;

/// Give [`Action`]s deterministic type and wrap them in [`Rc`] to make them clone-able.
///
/// When constructing a lexer using [`LexerBuilder`](crate::lexer::builder::LexerBuilder)
/// users should use [`Action`] to represent
/// both immutable and mutable actions, so that [`LexerBuilder`](crate::lexer::builder::LexerBuilder)
/// can accept one or more actions in one method call.
/// But when the lexer is built, to optimize the runtime performance,
/// we should know the exact type of each action instead of using pattern matching
/// to determine the type every time, and collect as many immutable actions as possible
/// to optimize stateless lexer's performance.
pub(super) enum GeneralAction<Kind: 'static, State, ErrorType> {
  Immutable(Rc<ImmutableAction<Kind, State, ErrorType>>),
  Mutable(Rc<MutableAction<Kind, State, ErrorType>>),
}

impl<Kind: 'static, State, ErrorType> Clone for GeneralAction<Kind, State, ErrorType> {
  #[inline]
  fn clone(&self) -> Self {
    match self {
      GeneralAction::Immutable(action) => GeneralAction::Immutable(action.clone()),
      GeneralAction::Mutable(action) => GeneralAction::Mutable(action.clone()),
    }
  }
}

impl<Kind, State, ErrorType> Action<Kind, State, ErrorType> {
  /// Convert [`Action`] into [`GeneralAction`].
  #[inline]
  pub(super) fn into_general(self) -> GeneralAction<Kind, State, ErrorType> {
    macro_rules! convert_action {
      ($action: ident, $exec: expr) => {
        Rc::new($action {
          kind: self.kind,
          literal: self.literal,
          head: self.head,
          muted: self.muted,
          exec: $exec,
        })
      };
    }

    match self.exec {
      ActionExec::Immutable(e) => GeneralAction::Immutable(convert_action!(ImmutableAction, e)),
      ActionExec::Mutable(e) => GeneralAction::Mutable(convert_action!(MutableAction, e)),
    }
  }
}

// re-export getters
macro_rules! re_export {
  ($self:expr, $name: ident) => {
    match $self {
      GeneralAction::Immutable(action) => action.$name(),
      GeneralAction::Mutable(action) => action.$name(),
    }
  };
}
impl<Kind: 'static, State, ErrorType> GeneralAction<Kind, State, ErrorType> {
  /// See [`Action::kind`].
  #[inline]
  pub fn kind(&self) -> &'static TokenKindId<Kind> {
    re_export!(self, kind)
  }
  /// See [`Action::literal`].
  #[inline]
  pub fn literal(&self) -> &Option<String> {
    re_export!(self, literal)
  }
  /// See [`Action::head`].
  #[inline]
  pub fn head(&self) -> &Option<HeadMatcher> {
    re_export!(self, head)
  }
  /// See [`Action::muted`].
  #[inline]
  pub fn muted(&self) -> bool {
    re_export!(self, muted)
  }
}

/// Conditionally convert [`ActionInput<&mut State>`] to [`ActionInput<&State>`].
///
/// Usage:
/// - `action_input_to_ref!(input, true)`: `&input.as_ref()`
/// - `action_input_to_ref!(input, false)`: `input`
macro_rules! mut_input_to_ref {
  ($input: ident, true) => {
    &$input.as_ref()
  };
  ($input: ident, false) => {
    $input
  };
}
pub(super) use mut_input_to_ref;

/// Convert the content of [`ActionExec`] using the given macro.
macro_rules! map_exec {
  ($exec: expr, $macro_impl: ident) => {
    match $exec {
      ActionExec::Immutable(exec) => ActionExec::Immutable($macro_impl!(exec)),
      ActionExec::Mutable(exec) => ActionExec::Mutable($macro_impl!(exec)),
    }
  };
}
pub(super) use map_exec;

/// Convert the content of [`ActionExec`] using the given macro.
/// The `macro_impl`'s second argument is a boolean indicating
/// whether to convert the mutable action to immutable.
macro_rules! map_exec_adapt_input {
  ($exec: expr, $macro_impl: ident) => {
    match $exec {
      ActionExec::Immutable(exec) => ActionExec::Immutable($macro_impl!(exec, false)),
      ActionExec::Mutable(exec) => ActionExec::Mutable($macro_impl!(exec, true)),
    }
  };
}
pub(super) use map_exec_adapt_input;

// helpers for tests
#[cfg(test)]
impl<Kind, State, ErrorType> ActionExec<Kind, State, ErrorType> {
  /// Try to convert [`ActionExec`] into [`ImmutableActionExec`].
  /// This is only for testing.
  /// # Panics
  /// If the action is mutable.
  pub(super) const fn as_immutable(&self) -> &ImmutableActionExec<Kind, State, ErrorType> {
    match self {
      ActionExec::Immutable(exec) => exec,
      ActionExec::Mutable(_) => panic!("ActionExec is mutable"),
    }
  }
  /// Try to convert [`ActionExec`] into [`MutableActionExec`].
  /// This is only for testing.
  /// # Panics
  /// If the action is immutable.
  pub(super) fn as_mutable(&self) -> &MutableActionExec<Kind, State, ErrorType> {
    match self {
      ActionExec::Immutable(_) => panic!("ActionExec is immutable"),
      ActionExec::Mutable(exec) => exec,
    }
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
      exec: ActionExec::Immutable(Box::new(|_| None)),
      kind: A::kind_id(),
      head: None,
      muted: false,
      literal: None,
    };
    assert!(!action.muted());
    assert_eq!(action.kind(), A::kind_id());
    assert!(action.head().is_none());
    assert!(action.literal().is_none());
  }

  #[test]
  fn action_getters() {
    let action: Action<_> = Action {
      exec: ActionExec::Immutable(Box::new(|_| None)),
      kind: A::kind_id(),
      head: Some(HeadMatcher::OneOf(HashSet::from(['a']))),
      muted: true,
      literal: Some("123".into()),
    };
    assert!(action.muted());
    assert_eq!(action.kind(), A::kind_id());
    assert!(matches!(action.head(), Some(HeadMatcher::OneOf(set)) if set == &HashSet::from(['a'])));
    assert_eq!(action.literal(), &Some("123".into()));
  }
}
