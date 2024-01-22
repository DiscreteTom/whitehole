use super::{action::Action, token::TokenKind, Lexer};

pub struct Builder<Kind: 'static, ActionState: 'static, ErrorType: 'static>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  actions: Vec<Action<Kind, ActionState, ErrorType>>,
}

impl<Kind, ActionState, ErrorType> Default for Builder<Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  fn default() -> Self {
    Builder {
      actions: Vec::new(),
    }
  }
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static> Builder<Kind, ActionState, ErrorType>
where
  Kind: TokenKind,
  ActionState: Clone + Default,
{
  pub fn append(mut self, action: Action<Kind, ActionState, ErrorType>) -> Self {
    self.actions.push(action);
    self
  }

  /// Define muted action.
  pub fn ignore(mut self, action: Action<Kind, ActionState, ErrorType>) -> Self {
    self.actions.push(action.mute(true));
    self
  }

  pub fn build<'buffer>(
    self,
    buffer: &'buffer str,
  ) -> Lexer<'buffer, Kind, ActionState, ErrorType> {
    Lexer::new(self.actions, buffer)
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
  fn append() {
    let mut lexer: Lexer<MyKind, (), ()> = Builder::default()
      .append(Action::regex("a+").unwrap().bind(MyKind::UnitField))
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

  #[test]
  fn ignore() {
    let mut lexer: Lexer<MyKind, (), ()> = Builder::default()
      .ignore(Action::regex("a+").unwrap().bind(MyKind::UnitField))
      .build("aaa");

    let res = lexer.lex();
    assert_eq!(res.digested, 3);
    assert_eq!(res.errors.len(), 0);
    assert!(res.token.is_none());
  }
}
