use super::{action::Action, token::TokenKind, Lexer};

pub struct Builder<Kind: 'static, ActionState: 'static, ErrorType: 'static> {
  actions: Vec<Action<Kind, ActionState, ErrorType>>,
  initial_state: ActionState,
}

impl<Kind, ActionState, ErrorType> Default for Builder<Kind, ActionState, ErrorType>
where
  ActionState: Default,
{
  fn default() -> Self {
    Builder {
      actions: Vec::new(),
      initial_state: ActionState::default(),
    }
  }
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static> Builder<Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone,
{
  pub fn new(state: ActionState) -> Builder<Kind, ActionState, ErrorType> {
    Builder {
      actions: Vec::new(),
      initial_state: state,
    }
  }

  pub fn define(mut self, action: Action<Kind, ActionState, ErrorType>) -> Self {
    self.actions.push(action);
    self
  }

  pub fn build<'buffer>(
    self,
    buffer: &'buffer str,
  ) -> Lexer<'buffer, Kind, ActionState, ErrorType> {
    Lexer::new(self.actions, self.initial_state, buffer)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use whitehole_macros::TokenKind;

  #[derive(TokenKind, Clone)]
  enum MyKind {
    UnitField,
    // UnnamedField(i32),
    // NamedField { _a: i32 },
  }

  #[test]
  fn simple_lexer_builder() {
    let mut lexer: Lexer<MyKind, (), ()> = Builder::new(())
      .define(Action::regex("a+").unwrap().bind(MyKind::UnitField))
      .build("aaa");

    let res = lexer.lex();
    assert_eq!(res.digested, 3);
    assert_eq!(res.errors.len(), 0);
    assert!(res.token.is_some());
    let token = res.token.unwrap();
    assert!(matches!(token.kind, MyKind::UnitField));
    assert_eq!(token.start, 0);
    assert_eq!(token.end, 3);
    assert_eq!(token.content(), "aaa");
    assert_eq!(token.error, None);
  }
}
