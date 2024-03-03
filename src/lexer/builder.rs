mod action_list;
mod append;
mod define;
mod ignore;

pub use action_list::*;

use super::{
  action::{Action, ActionInputRestHeadMatcher},
  stateless::{ActionHeadMap, StatelessLexer},
  token::TokenKind,
  Lexer,
};
use std::{collections::HashMap, rc::Rc};

pub struct LexerBuilder<Kind, ActionState = (), ErrorType = ()> {
  actions: Vec<Action<Kind, ActionState, ErrorType>>,
}

impl<Kind, ActionState, ErrorType> Default for LexerBuilder<Kind, ActionState, ErrorType> {
  fn default() -> Self {
    Self {
      actions: Vec::new(),
    }
  }
}
impl<Kind, ActionState, ErrorType> From<Vec<Action<Kind, ActionState, ErrorType>>>
  for LexerBuilder<Kind, ActionState, ErrorType>
{
  fn from(actions: Vec<Action<Kind, ActionState, ErrorType>>) -> Self {
    Self { actions }
  }
}
impl<Kind, ActionState, ErrorType, const N: usize> From<[Action<Kind, ActionState, ErrorType>; N]>
  for LexerBuilder<Kind, ActionState, ErrorType>
{
  fn from(actions: [Action<Kind, ActionState, ErrorType>; N]) -> Self {
    Self {
      actions: actions.into(),
    }
  }
}
impl<
    Kind: TokenKind<Kind> + Clone + 'static,
    ActionState: 'static,
    ErrorType: 'static,
    const N: usize,
  > From<[(Kind, ActionList<Action<(), ActionState, ErrorType>>); N]>
  for LexerBuilder<Kind, ActionState, ErrorType>
{
  fn from(actions: [(Kind, ActionList<Action<(), ActionState, ErrorType>>); N]) -> Self {
    Self::default().define_from(actions)
  }
}

impl<Kind, ActionState, ErrorType> LexerBuilder<Kind, ActionState, ErrorType> {
  pub fn new() -> Self {
    Self::default()
  }

  // TODO: move into `generate`?
  pub fn build_stateless(self) -> StatelessLexer<Kind, ActionState, ErrorType> {
    // wrap actions in Rc and make them immutable
    let actions = self.actions.into_iter().map(Rc::new).collect::<Vec<_>>();

    let mut kind_map = HashMap::new();
    // prepare kind map, add value for all possible kinds
    for a in &actions {
      for k in a.possible_kinds() {
        kind_map.entry(k.clone()).or_insert(Vec::new());
      }
    }
    // fill kind_map
    for a in &actions {
      if a.maybe_muted {
        // maybe muted, add to all kinds
        for (_, vec) in kind_map.iter_mut() {
          vec.push(a.clone());
        }
      } else {
        // never muted, only add to possible kinds
        for k in a.possible_kinds() {
          kind_map.get_mut(k).unwrap().push(a.clone());
        }
      }
    }
    // the above code should make sure the order of actions in each vec is the same as the order in `actions`

    let maybe_muted_actions = actions
      .iter()
      .filter(|a| a.maybe_muted)
      .map(|a| a.clone())
      .collect();

    let kind_head_map = kind_map
      .iter()
      .map(|(k, v)| (k.clone(), Self::calc_head_map(&v)))
      .collect();
    let head_map = Self::calc_head_map(&actions);
    let maybe_muted_head_map = Self::calc_head_map(&maybe_muted_actions);

    StatelessLexer::new(actions, head_map, kind_head_map, maybe_muted_head_map)
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

  fn calc_head_map(
    actions: &Vec<Rc<Action<Kind, ActionState, ErrorType>>>,
  ) -> ActionHeadMap<Kind, ActionState, ErrorType> {
    let mut head_map = ActionHeadMap {
      known_map: HashMap::new(),
      unknown_fallback: Vec::new(),
    };
    // collect all known chars
    for a in actions {
      if let Some(head_matcher) = a.head_matcher() {
        for c in match head_matcher {
          ActionInputRestHeadMatcher::OneOf(set) => set,
          ActionInputRestHeadMatcher::Not(set) => set,
          ActionInputRestHeadMatcher::Unknown => continue,
        } {
          head_map.known_map.entry(*c).or_insert(Vec::new());
        }
      }
    }
    // fill the head_map
    for a in actions {
      if let Some(head_matcher) = a.head_matcher() {
        match head_matcher {
          ActionInputRestHeadMatcher::OneOf(set) => {
            for c in set {
              head_map.known_map.get_mut(c).unwrap().push(a.clone());
            }
          }
          ActionInputRestHeadMatcher::Not(set) => {
            for (c, vec) in head_map.known_map.iter_mut() {
              if !set.contains(c) {
                vec.push(a.clone());
              }
            }
            head_map.unknown_fallback.push(a.clone());
          }
          ActionInputRestHeadMatcher::Unknown => {
            head_map.unknown_fallback.push(a.clone());
          }
        }
      } else {
        // no head matcher, add to all known chars
        for vec in head_map.known_map.values_mut() {
          vec.push(a.clone());
        }
        // and unknown fallback
        head_map.unknown_fallback.push(a.clone());
      }
    }
    // the above code should make sure the order of actions in each vec is the same as the order in `actions`

    head_map
  }

  fn map_actions<OldKind: 'static, NewKind, F>(
    actions: impl Into<ActionList<Action<OldKind, ActionState, ErrorType>>>,
    f: F,
  ) -> Vec<Action<NewKind, ActionState, ErrorType>>
  where
    F: Fn(Action<OldKind, ActionState, ErrorType>) -> Action<NewKind, ActionState, ErrorType>,
  {
    actions.into().0.into_iter().map(f).collect::<Vec<_>>()
  }
}

impl<Kind, ActionState, ErrorType> Into<StatelessLexer<Kind, ActionState, ErrorType>>
  for LexerBuilder<Kind, ActionState, ErrorType>
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
      .append_with(|a| a.regex("a+").unwrap().bind(MyKind::UnitField).into())
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
        .into()
    });
  }

  #[derive(Clone, Default)]
  struct MyState {
    pub reject: bool,
  }
}
