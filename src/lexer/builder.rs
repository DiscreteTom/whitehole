use super::{
  action::{builder::ActionBuilder, Action},
  stateless::StatelessLexer,
  token::TokenKind,
  Lexer,
};
use std::rc::Rc;

// impl Into<Vec> for Action so that the builder can accept one or multiple actions
impl<Kind: 'static, ActionState: 'static, ErrorType: 'static>
  Into<Vec<Action<Kind, ActionState, ErrorType>>> for Action<Kind, ActionState, ErrorType>
{
  fn into(self) -> Vec<Action<Kind, ActionState, ErrorType>> {
    vec![self]
  }
}

pub struct LexerBuilder<Kind: 'static, ActionState: 'static = (), ErrorType: 'static = ()>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  actions: Vec<Action<Kind, ActionState, ErrorType>>,
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static>
  LexerBuilder<Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind> + Default + Clone,
  ActionState: Clone + Default,
{
  /// Define [muted](Action::maybe_muted) actions.
  /// ```
  /// # use whitehole::lexer::{action::{Action, whitespaces}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// // make sure `Default` is implemented
  /// #[derive(TokenKind, Clone, Default)]
  /// enum MyKind {
  ///   #[default]
  ///   Anonymous,
  /// }
  /// LexerBuilder::<MyKind>::default()
  ///   // single action
  ///   .ignore_default(whitespaces())
  ///   // multiple actions
  ///   .ignore_default([whitespaces(), whitespaces()]);
  /// ```
  pub fn ignore_default(self, actions: impl Into<Vec<Action<(), ActionState, ErrorType>>>) -> Self {
    self.ignore(Self::map_actions(actions, |a| a.bind(Kind::default())))
  }

  /// Define [muted](Action::maybe_muted) action with [`ActionBuilder`].
  /// ```
  /// # use whitehole::lexer::{action::Action, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// // make sure `Default` is implemented
  /// #[derive(TokenKind, Clone, Default)]
  /// enum MyKind {
  ///   #[default]
  ///   Anonymous,
  /// }
  /// let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.ignore_default_with(|a| a.regex(r"\s+").unwrap());
  /// ```
  pub fn ignore_default_with<F>(self, factory: F) -> Self
  where
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<(), ActionState, ErrorType>,
  {
    self.ignore_default(factory(ActionBuilder::default()))
  }

  /// Define [muted](Action::maybe_muted) action.
  pub fn ignore_default_from<F, const N: usize>(self, factory_vec: [F; N]) -> Self
  where
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<(), ActionState, ErrorType>,
  {
    factory_vec
      .into_iter()
      .fold(self, |builder, f| builder.ignore_default_with(f))
  }

  pub fn append_default(self, actions: impl Into<Vec<Action<(), ActionState, ErrorType>>>) -> Self {
    self.append(Self::map_actions(actions, |a| a.bind(Kind::default())))
  }

  pub fn append_default_with<F>(self, factory: F) -> Self
  where
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<(), ActionState, ErrorType>,
  {
    self.append_default(factory(ActionBuilder::default()))
  }

  pub fn append_default_from<F, const N: usize>(self, factory_vec: [F; N]) -> Self
  where
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<(), ActionState, ErrorType>,
  {
    factory_vec
      .into_iter()
      .fold(self, |builder, f| builder.append_default_with(f))
  }
}

impl<Kind, ActionState, ErrorType> Default for LexerBuilder<Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  fn default() -> Self {
    LexerBuilder {
      actions: Vec::new(),
    }
  }
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static>
  LexerBuilder<Kind, ActionState, ErrorType>
where
  Kind: TokenKind<Kind>,
  ActionState: Clone + Default,
{
  fn map_actions<OldKind: 'static, NewKind, F>(
    actions: impl Into<Vec<Action<OldKind, ActionState, ErrorType>>>,
    f: F,
  ) -> Vec<Action<NewKind, ActionState, ErrorType>>
  where
    F: Fn(Action<OldKind, ActionState, ErrorType>) -> Action<NewKind, ActionState, ErrorType>,
  {
    actions.into().into_iter().map(f).collect::<Vec<_>>()
  }

  pub fn append(mut self, actions: impl Into<Vec<Action<Kind, ActionState, ErrorType>>>) -> Self {
    actions
      .into()
      .into_iter()
      .for_each(|action| self.actions.push(action));
    self
  }

  pub fn append_with<F>(self, factory: F) -> Self
  where
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<Kind, ActionState, ErrorType>,
  {
    self.append(factory(ActionBuilder::default()))
  }

  pub fn append_from<F, const N: usize>(self, factory_vec: [F; N]) -> Self
  where
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<Kind, ActionState, ErrorType>,
  {
    factory_vec
      .into_iter()
      .fold(self, |builder, f| builder.append_with(f))
  }

  pub fn define(
    self,
    kind: impl Into<Kind>,
    actions: impl Into<Vec<Action<(), ActionState, ErrorType>>>,
  ) -> Self
  where
    Kind: Clone,
  {
    let kind = kind.into();
    self.append(Self::map_actions(actions, |a| a.bind(kind.clone())))
  }

  pub fn from(
    self,
    defs: Vec<(
      impl Into<Kind>,
      impl Into<Vec<Action<(), ActionState, ErrorType>>>,
    )>,
  ) -> Self
  where
    Kind: Clone,
  {
    defs.into_iter().fold(self, |builder, (kind, actions)| {
      builder.define(kind, actions)
    })
  }

  pub fn define_with<F>(self, kind: impl Into<Kind>, factory: F) -> Self
  where
    Kind: Clone,
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<(), ActionState, ErrorType>,
  {
    self.define(kind, factory(ActionBuilder::default()))
  }

  pub fn define_from<F, const N: usize>(self, kind: impl Into<Kind>, factory_vec: [F; N]) -> Self
  where
    Kind: Clone,
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<(), ActionState, ErrorType>,
  {
    let kind = kind.into();
    factory_vec
      .into_iter()
      .fold(self, |builder, f| builder.define_with(kind.clone(), f))
  }

  /// Define [muted](Action::maybe_muted) action.
  pub fn ignore(self, actions: impl Into<Vec<Action<Kind, ActionState, ErrorType>>>) -> Self {
    self.append(Self::map_actions(actions, |a| a.mute(true)))
  }

  /// Define [muted](Action::maybe_muted) action.
  pub fn ignore_with<F>(self, factory: F) -> Self
  where
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<Kind, ActionState, ErrorType>,
  {
    self.ignore(factory(ActionBuilder::default()))
  }

  /// Define [muted](Action::maybe_muted) action.
  pub fn ignore_from<F, const N: usize>(self, factory_vec: [F; N]) -> Self
  where
    F: FnOnce(ActionBuilder<ActionState, ErrorType>) -> Action<Kind, ActionState, ErrorType>,
  {
    factory_vec
      .into_iter()
      .fold(self, |builder, f| builder.ignore_with(f))
  }

  pub fn build<'text>(self, text: &'text str) -> Lexer<'text, Kind, ActionState, ErrorType> {
    Lexer::new(Rc::new(self.build_stateless()), text)
  }

  pub fn build_stateless(self) -> StatelessLexer<Kind, ActionState, ErrorType> {
    StatelessLexer::new(self.actions)
  }
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
