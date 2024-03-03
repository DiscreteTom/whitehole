use super::{
  action::{Action, ActionBuilder},
  stateless::StatelessLexer,
  token::TokenKind,
  Lexer,
};
use std::rc::Rc;

/// A helper struct to accept one or more actions.
pub struct ActionList<Action>(pub Vec<Action>);
impl<Kind, ActionState, ErrorType> From<Action<Kind, ActionState, ErrorType>>
  for ActionList<Action<Kind, ActionState, ErrorType>>
{
  fn from(value: Action<Kind, ActionState, ErrorType>) -> Self {
    Self(vec![value])
  }
}
impl<Kind, ActionState, ErrorType> From<Vec<Action<Kind, ActionState, ErrorType>>>
  for ActionList<Action<Kind, ActionState, ErrorType>>
{
  fn from(value: Vec<Action<Kind, ActionState, ErrorType>>) -> Self {
    Self(value)
  }
}
impl<Kind, ActionState, ErrorType, const N: usize> From<[Action<Kind, ActionState, ErrorType>; N]>
  for ActionList<Action<Kind, ActionState, ErrorType>>
{
  fn from(value: [Action<Kind, ActionState, ErrorType>; N]) -> Self {
    Self(value.into())
  }
}

pub struct LexerBuilder<Kind, ActionState = (), ErrorType = ()> {
  actions: Vec<Action<Kind, ActionState, ErrorType>>,
}

impl<Kind, ActionState, ErrorType> Default for LexerBuilder<Kind, ActionState, ErrorType> {
  fn default() -> Self {
    LexerBuilder {
      actions: Vec::new(),
    }
  }
}

impl<Kind, ActionState, ErrorType> LexerBuilder<Kind, ActionState, ErrorType> {
  // TODO: add new as an alias of default

  fn map_actions<OldKind: 'static, NewKind, F>(
    actions: impl Into<ActionList<Action<OldKind, ActionState, ErrorType>>>,
    f: F,
  ) -> Vec<Action<NewKind, ActionState, ErrorType>>
  where
    F: Fn(Action<OldKind, ActionState, ErrorType>) -> Action<NewKind, ActionState, ErrorType>,
  {
    actions.into().0.into_iter().map(f).collect::<Vec<_>>()
  }

  /// Append actions to the builder.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append a single action
  /// builder.append(word("A").bind(A));
  /// // append multiple actions
  /// builder.append([word("A").bind(A), word("B").bind(B)]);
  /// ```
  pub fn append(
    mut self,
    actions: impl Into<ActionList<Action<Kind, ActionState, ErrorType>>>,
  ) -> Self {
    actions
      .into()
      .0
      .into_iter()
      .for_each(|action| self.actions.push(action));
    self
  }

  /// Append an action with [`ActionBuilder`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.append_with(|a| a.from(word("A")).bind(A));
  /// ```
  pub fn append_with<F>(self, factory: F) -> Self
  where
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<Kind, ActionState, ErrorType>,
  {
    self.append(factory(ActionBuilder::default()))
  }

  // TODO: rename?
  /// Append actions with a list of [`ActionBuilder`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.append_from([
  ///   |a| a.from(word("A")).bind(A),
  ///   |a| a.from(word("B")).bind(B)
  /// ]);
  /// ```
  pub fn append_from<F, const N: usize>(self, factory_vec: [F; N]) -> Self
  where
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<Kind, ActionState, ErrorType>,
  {
    factory_vec
      .into_iter()
      .fold(self, |builder, f| builder.append_with(f))
  }

  /// Append actions and bind them to the default kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, whitespaces, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Default, Clone)]
  /// # enum MyKind {
  /// #   #[default]
  /// #   Anonymous,
  /// # }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append a single action
  /// builder.append_default(whitespaces());
  /// // append multiple actions
  /// builder.append_default([whitespaces(), word("_")]);
  /// ```
  pub fn append_default(
    self,
    actions: impl Into<ActionList<Action<(), ActionState, ErrorType>>>,
  ) -> Self
  where
    Kind: TokenKind<Kind> + Default + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    self.append(Self::map_actions(actions, |a| a.bind(Kind::default())))
  }

  /// Append an action with [`ActionBuilder`] and bind it to the default kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, whitespaces, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Default, Clone)]
  /// # enum MyKind {
  /// #   #[default]
  /// #   Anonymous,
  /// # }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.append_default_with(|a| a.from(whitespaces()));
  /// ```
  pub fn append_default_with<F>(self, factory: F) -> Self
  where
    Kind: TokenKind<Kind> + Default + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<(), ActionState, ErrorType>,
  {
    self.append_default(factory(ActionBuilder::default()))
  }

  /// Append actions with a list of [`ActionBuilder`] and bind them to the default kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, whitespaces, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Default, Clone)]
  /// # enum MyKind {
  /// #   #[default]
  /// #   Anonymous,
  /// # }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.append_default_from([
  ///   |a| a.from(word("A")),
  ///   |a| a.from(word("B")),
  /// ]);
  /// ```
  pub fn append_default_from<F, const N: usize>(self, factory_vec: [F; N]) -> Self
  where
    Kind: TokenKind<Kind> + Default + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<(), ActionState, ErrorType>,
  {
    factory_vec
      .into_iter()
      .fold(self, |builder, f| builder.append_default_with(f))
  }

  /// Define [`muted`](Action::maybe_muted) actions by calling [`Action::mute`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append a single action
  /// builder.ignore(word("A").bind(A));
  /// // append multiple actions
  /// builder.ignore([word("A").bind(A), word("B").bind(B)]);
  /// ```
  pub fn ignore(self, actions: impl Into<ActionList<Action<Kind, ActionState, ErrorType>>>) -> Self
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    self.append(Self::map_actions(actions, |a| a.mute(true)))
  }

  /// Define a [`muted`](Action::maybe_muted) action by calling [`Action::mute`] with [`ActionBuilder`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.ignore_with(|a| a.from(word("A")).bind(A));
  /// ```
  pub fn ignore_with<F>(self, factory: F) -> Self
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<Kind, ActionState, ErrorType>,
  {
    self.ignore(factory(ActionBuilder::default()))
  }

  /// Define [`muted`](Action::maybe_muted) actions by calling [`Action::mute`] with a list of [`ActionBuilder`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.append_from([
  ///   |a| a.from(word("A")).bind(A),
  ///   |a| a.from(word("B")).bind(B)
  /// ]);
  /// ```
  pub fn ignore_from<F, const N: usize>(self, factory_vec: [F; N]) -> Self
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<Kind, ActionState, ErrorType>,
  {
    factory_vec
      .into_iter()
      .fold(self, |builder, f| builder.ignore_with(f))
  }

  /// Define [`muted`](Action::maybe_muted) actions by calling [`Action::mute`] and bind them to the default kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, whitespaces, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Default, Clone)]
  /// # enum MyKind {
  /// #   #[default]
  /// #   Anonymous,
  /// # }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append a single action
  /// builder.ignore_default(whitespaces());
  /// // append multiple actions
  /// builder.ignore_default([whitespaces(), word("_")]);
  /// ```
  pub fn ignore_default(
    self,
    actions: impl Into<ActionList<Action<(), ActionState, ErrorType>>>,
  ) -> Self
  where
    Kind: TokenKind<Kind> + Default + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    self.ignore(Self::map_actions(actions, |a| a.bind(Kind::default())))
  }

  /// Define a [`muted`](Action::maybe_muted) action by calling [`Action::mute`] with [`ActionBuilder`] and bind it to the default kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, whitespaces, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Default, Clone)]
  /// # enum MyKind {
  /// #   #[default]
  /// #   Anonymous,
  /// # }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.ignore_default_with(|a| a.from(whitespaces()));
  /// ```
  pub fn ignore_default_with<F>(self, factory: F) -> Self
  where
    Kind: TokenKind<Kind> + Default + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<(), ActionState, ErrorType>,
  {
    self.ignore_default(factory(ActionBuilder::default()))
  }

  /// Define [`muted`](Action::maybe_muted) actions by calling [`Action::mute`] with a list of [`ActionBuilder`] and bind them to the default kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, whitespaces, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Default, Clone)]
  /// # enum MyKind {
  /// #   #[default]
  /// #   Anonymous,
  /// # }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.ignore_default_from([
  ///   |a| a.from(word("A")).bind(A),
  ///   |a| a.from(word("B")).bind(B)
  /// ]);
  /// ```
  pub fn ignore_default_from<F, const N: usize>(self, factory_vec: [F; N]) -> Self
  where
    Kind: TokenKind<Kind> + Default + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<(), ActionState, ErrorType>,
  {
    factory_vec
      .into_iter()
      .fold(self, |builder, f| builder.ignore_default_with(f))
  }

  /// Define actions and bind them to the provided kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// // append a single action
  /// builder.define(A, word("A"));
  /// // append multiple actions
  /// builder.define(A, [word("A"), word("AA")]);
  /// ```
  pub fn define(
    self,
    kind: impl Into<Kind>,
    actions: impl Into<ActionList<Action<(), ActionState, ErrorType>>>,
  ) -> Self
  where
    Kind: TokenKind<Kind> + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    let kind = kind.into();
    self.append(Self::map_actions(actions, |a| a.bind(kind.clone())))
  }

  /// Define an action with [`ActionBuilder`] and bind it to the provided kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define_with(A, |a| a.from(word("A")));
  /// ```
  pub fn define_with<F>(self, kind: impl Into<Kind>, factory: F) -> Self
  where
    Kind: TokenKind<Kind> + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<(), ActionState, ErrorType>,
  {
    self.define(kind, factory(ActionBuilder::default()))
  }

  /// Define actions with a list of [`ActionBuilder`] and bind them to the provided kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, word}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # use MyKind::*;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A, B }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define_from(A, [
  ///   |a| a.from(word("A")).bind(A),
  ///   |a| a.from(word("B")).bind(B)
  /// ]);
  /// ```
  pub fn define_from<F, const N: usize>(self, kind: impl Into<Kind>, factory_vec: [F; N]) -> Self
  where
    Kind: TokenKind<Kind> + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<(), ActionState, ErrorType>,
  {
    let kind = kind.into();
    factory_vec
      .into_iter()
      .fold(self, |builder, f| builder.define_with(kind.clone(), f))
  }

  pub fn build_stateless(self) -> StatelessLexer<Kind, ActionState, ErrorType> {
    StatelessLexer::new(self.actions)
  }

  pub fn build<'text>(self, text: &'text str) -> Lexer<'text, Kind, ActionState, ErrorType>
  where
    ActionState: Default,
  {
    Lexer::with_default_action_state(Rc::new(self.build_stateless()), text)
  }
  // TODO: add build_with
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static>
  Into<StatelessLexer<Kind, ActionState, ErrorType>> for LexerBuilder<Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  fn into(self) -> StatelessLexer<Kind, ActionState, ErrorType> {
    self.build_stateless()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::regex;
  use whitehole_macros::_TokenKind;

  #[derive(_TokenKind, Clone)]
  enum MyKind {
    UnitField,
    // UnnamedField(i32),
    // NamedField { _a: i32 },
  }

  #[test]
  fn append() {
    let mut lexer: Lexer<MyKind, (), ()> = LexerBuilder::default()
      .append_with(|a| a.regex("a+").unwrap().bind(MyKind::UnitField))
      .build("aaa");

    let res = lexer.lex();
    assert_eq!(res.digested, 3);
    assert_eq!(res.errors.len(), 0);
    assert!(res.token.is_some());
    let token = res.token.unwrap();
    assert!(matches!(token.kind, MyKind::UnitField));
    assert_eq!(token.range.start, 0);
    assert_eq!(token.range.end, 3);
    assert_eq!(token.content, "aaa");
    assert!(matches!(token.error, None));
  }

  #[test]
  fn ignore() {
    let mut lexer: Lexer<MyKind, (), ()> = LexerBuilder::default()
      .ignore(regex("a+").unwrap().bind(MyKind::UnitField))
      .build("aaa");

    let res = lexer.lex();
    assert_eq!(res.digested, 3);
    assert_eq!(res.errors.len(), 0);
    assert!(res.token.is_none());

    LexerBuilder::<MyKind, MyState>::default().define_with(MyKind::UnitField, |a| {
      a.regex(r"^\s+")
        .unwrap()
        .prevent(|input| input.state.reject)
    });
  }

  #[derive(Clone, Default)]
  struct MyState {
    pub reject: bool,
  }
}
