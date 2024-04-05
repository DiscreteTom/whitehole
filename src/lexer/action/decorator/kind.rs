use super::AcceptedActionOutputContext;
use crate::lexer::{
  action::{Action, ActionInput, ActionOutput},
  token::{DefaultTokenKindIdBinding, SubTokenKind, TokenKindIdBinding},
};

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  /// Set the kind and the data binding for this action.
  /// Use this if your action can only yield a const token kind value.
  /// # Examples
  /// ```
  /// use whitehole::lexer::action::{Action, simple};
  /// use whitehole::lexer::token::TokenKindIdBinding;
  /// use whitehole_macros::TokenKind;
  ///
  /// #[derive(TokenKind, Clone, Debug)]
  /// enum MyKind { A }
  ///
  /// let action: Action<TokenKindIdBinding<MyKind>> = simple(|_| 1).bind(A);
  /// ```
  pub fn bind<NewKind, ViaKind>(
    self,
    kind: ViaKind,
  ) -> Action<TokenKindIdBinding<NewKind>, ActionState, ErrorType>
  where
    ViaKind: SubTokenKind<TokenKindIdBinding<NewKind>> + Into<TokenKindIdBinding<NewKind>>,
    NewKind: Clone + 'static,
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    let exec = self.exec;
    let kind = kind.into();
    Action {
      kind_id: ViaKind::kind_id(),
      head_matcher: self.head_matcher,
      muted: self.muted,
      may_mutate_state: self.may_mutate_state,
      literal: self.literal,
      exec: Box::new(move |input| {
        exec(input).map(|output| ActionOutput {
          kind: kind.clone(),
          digested: output.digested,
          error: output.error,
        })
      }),
    }
  }

  /// Set the kind to the default for this action.
  /// # Examples
  /// ```
  /// use whitehole::lexer::action::{Action, simple};
  /// use whitehole::lexer::token::TokenKindIdBinding;
  /// use whitehole_macros::TokenKind;
  ///
  /// #[derive(TokenKind, Clone, Debug, Default)]
  /// enum MyKind { #[default] Anonymous, A }
  ///
  /// let action: Action<TokenKindIdBinding<MyKind>> = simple(|_| 1).bind_default();
  /// ```
  pub fn bind_default<NewKind>(self) -> Action<TokenKindIdBinding<NewKind>, ActionState, ErrorType>
  where
    NewKind: Default + DefaultTokenKindIdBinding<NewKind>,
    ActionState: 'static,
    ErrorType: 'static,
  {
    let exec = self.exec;
    Action {
      kind_id: NewKind::default_binding_kind_id(),
      head_matcher: self.head_matcher,
      muted: self.muted,
      may_mutate_state: self.may_mutate_state,
      literal: self.literal,
      exec: Box::new(move |input| {
        exec(input).map(|output| ActionOutput {
          kind: TokenKindIdBinding::default(),
          digested: output.digested,
          error: output.error,
        })
      }),
    }
  }

  /// Set the kind and the data binding for this action by the selector.
  /// Use this if you need to calculate the kind based on the [`ActionInput`] and [`ActionOutput`].
  /// # Examples
  /// ```
  /// use whitehole::lexer::action::{Action, regex};
  /// use whitehole::lexer::token::TokenKindIdBinding;
  /// use whitehole_macros::TokenKind;
  ///
  /// #[derive(TokenKind, Clone, Debug)]
  /// enum MyKind { Num(i32) }
  ///
  /// let action: Action<TokenKindIdBinding<MyKind>> = regex(r"^\d+")
  ///   .unwrap()
  ///   .select(|ctx| Num(ctx.content().parse().unwrap());
  /// ```
  pub fn select<NewKind, ViaKind>(
    self,
    selector: impl Fn(
        AcceptedActionOutputContext<
          // user can't mutate the input
          &ActionInput<ActionState>,
          // output is consumed except the error
          ActionOutput<Kind, &Option<ErrorType>>,
        >,
      ) -> ViaKind
      + 'static,
  ) -> Action<TokenKindIdBinding<NewKind>, ActionState, ErrorType>
  where
    ViaKind: Into<TokenKindIdBinding<NewKind>> + SubTokenKind<TokenKindIdBinding<NewKind>>,
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    let exec = self.exec;
    Action {
      kind_id: ViaKind::kind_id(),
      head_matcher: self.head_matcher,
      muted: self.muted,
      may_mutate_state: self.may_mutate_state,
      literal: self.literal,
      exec: Box::new(move |input| {
        exec(input).map(|output| {
          ActionOutput {
            kind: selector(AcceptedActionOutputContext {
              input,
              // construct a new ActionOutput
              output: ActionOutput {
                // consume the original output.kind
                kind: output.kind,
                digested: output.digested,
                // but don't consume the error
                error: &output.error,
              },
            })
            .into(),
            digested: output.digested,
            error: output.error,
          }
        })
      }),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::{
    action::{exact, regex},
    token::TokenKindIdProvider,
  };
  use whitehole_macros::_token_kind;

  #[_token_kind]
  #[derive(Clone, Debug, Default)]
  enum MyKind {
    #[default]
    A,
    Value(i32),
  }

  #[test]
  fn action_bind() {
    let action: Action<_> = exact("A").bind(A);
    assert_eq!(action.kind_id, A::kind_id());
    assert!(matches!(
      action.exec(&mut ActionInput::new("A", 0, &mut ()).unwrap()),
      Some(ActionOutput {
        kind: binding,
        digested: 1,
        error: None
      }) if matches!(binding.value(), MyKind::A) && binding.id() == A::kind_id()
    ));
  }

  #[test]
  fn action_bind_default() {
    let action: Action<_> = exact("A").bind_default();
    assert_eq!(action.kind_id, MyKind::default_binding_kind_id());
    assert!(matches!(
      action.exec(&mut ActionInput::new("A", 0, &mut ()).unwrap()),
      Some(ActionOutput {
        kind: binding,
        digested: 1,
        error: None
      }) if matches!(binding.value(), MyKind::A) && binding.id() == MyKind::default_binding_kind_id()
    ));
  }

  #[test]
  fn action_select() {
    let action: Action<_> =
      Action::from(regex(r"^\d+")).select(|ctx| Value(ctx.content().parse().unwrap()));
    assert_eq!(action.kind_id, Value::kind_id());
    assert!(matches!(
      action.exec(&mut ActionInput::new("1", 0, &mut ()).unwrap()),
      Some(ActionOutput {
        kind: binding,
        digested: 1,
        error: None
      }) if matches!(binding.value(), MyKind::Value(value) if value.0 == 1) && binding.id() == Value::kind_id()
    ));
  }
}
