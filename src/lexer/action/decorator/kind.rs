use super::AcceptedActionDecoratorContext;
use crate::lexer::{
  action::{ActionInput, ActionOutput, EnhancedActionOutput},
  token::{SubTokenKind, TokenKindId, TokenKindIdBinding},
  Action,
};

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  /// Set the kind and the data binding for this action.
  /// Use this if your action can only yield one kind.
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
      maybe_muted: self.maybe_muted,
      may_mutate_state: self.may_mutate_state,
      exec: Box::new(move |input| {
        exec(input).map(|output| ActionOutput {
          kind: kind.clone(),
          digested: output.digested,
          muted: output.muted,
          error: output.error,
        })
      }),
    }
  }

  /// Set the kind to the default for this action.
  /// The default kind must have `0` as its id.
  /// # Examples
  /// ```
  /// use whitehole::lexer::action::{Action, simple};
  /// use whitehole::lexer::token::TokenKindIdBinding;
  /// use whitehole_macros::TokenKind;
  ///
  /// // the default sub kind MUST be the first one
  /// // and annotated with `#[default]` provided by `Default` derive
  /// #[derive(TokenKind, Clone, Debug, Default)]
  /// enum MyKind { #[default] Anonymous, A }
  ///
  /// let action: Action<TokenKindIdBinding<MyKind>> = simple(|_| 1).bind_default();
  /// ```
  pub fn bind_default<NewKind>(self) -> Action<NewKind, ActionState, ErrorType>
  where
    NewKind: Default,
  {
    let exec = self.exec;
    Action {
      kind_id: TokenKindId::new(0),
      head_matcher: self.head_matcher,
      maybe_muted: self.maybe_muted,
      may_mutate_state: self.may_mutate_state,
      exec: Box::new(move |input| {
        exec(input).map(|output| ActionOutput {
          kind: NewKind::default(),
          digested: output.digested,
          muted: output.muted,
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
  ///   .select(|ctx| Num(ctx.output.content().parse().unwrap());
  /// ```
  pub fn select<NewKind, ViaKind, F>(
    self,
    selector: F,
  ) -> Action<TokenKindIdBinding<NewKind>, ActionState, ErrorType>
  where
    ViaKind: Into<TokenKindIdBinding<NewKind>> + SubTokenKind<TokenKindIdBinding<NewKind>>,
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(
        AcceptedActionDecoratorContext<
          // user can't mutate the input
          &ActionInput<ActionState>,
          // output is consumed except the error
          EnhancedActionOutput<Kind, &Option<ErrorType>>,
        >,
      ) -> ViaKind
      + 'static,
  {
    let exec = self.exec;
    Action {
      kind_id: ViaKind::kind_id(),
      head_matcher: self.head_matcher,
      maybe_muted: self.maybe_muted,
      may_mutate_state: self.may_mutate_state,
      exec: Box::new(move |input| {
        exec(input).map(|output| {
          ActionOutput {
            kind: selector(AcceptedActionDecoratorContext {
              output: EnhancedActionOutput::new(
                input,
                // construct a new ActionOutput
                ActionOutput {
                  // consume the original output.kind
                  kind: output.kind,
                  digested: output.digested,
                  muted: output.muted,
                  // but don't consume the error
                  error: &output.error,
                },
              ),
              input,
            })
            .into(),
            digested: output.digested,
            muted: output.muted,
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
    action::{regex, simple},
    token::TokenKindIdProvider,
  };
  use whitehole_macros::_TokenKind;

  #[derive(_TokenKind, Clone, Debug)]
  enum MyKind {
    A,
    Value(i32),
  }

  #[test]
  fn action_bind() {
    let action: Action<TokenKindIdBinding<MyKind>> = simple(|_| 1).bind(A);
    assert_eq!(action.kind_id, A::kind_id());
    assert!(matches!(
      action.exec(&mut ActionInput::new("A", 0, &mut ())),
      Some(ActionOutput {
        kind: binding,
        digested: 1,
        muted: false,
        error: None
      }) if matches!(binding.value(), MyKind::A) && binding.id() == &A::kind_id()
    ));
  }

  #[test]
  fn action_select() {
    let action: Action<TokenKindIdBinding<MyKind>> = regex(r"^\d+")
      .unwrap()
      .select(|ctx| Value(ctx.output.content().parse().unwrap()));
    assert_eq!(action.kind_id, Value::kind_id());
    assert!(matches!(
      action.exec(&mut ActionInput::new("1", 0, &mut ())),
      Some(ActionOutput {
        kind: binding,
        digested: 1,
        muted: false,
        error: None
      }) if matches!(binding.value(), MyKind::Value(value) if value == &1) && binding.id() == &Value::kind_id()
    ));
  }
}
