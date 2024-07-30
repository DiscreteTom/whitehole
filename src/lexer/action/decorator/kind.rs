use super::AcceptedActionOutputContext;
use crate::lexer::{
  action::{action_input_to_ref, Action, ActionExec, ActionInput, ActionOutput},
  token::{DefaultTokenKindIdBinding, SubTokenKind, TokenKindIdBinding},
};

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  /// Set the kind and the data binding for this action.
  /// Use this if your action can only yield a const token kind value.
  /// # Examples
  /// ```
  /// use whitehole::lexer::{
  ///   action::{Action, exact},
  ///   token::{TokenKindIdBinding, token_kind},
  /// };
  ///
  /// #[token_kind]
  /// #[derive(Clone, Debug)]
  /// enum MyKind { A }
  ///
  /// # fn main() {
  /// let action: Action<TokenKindIdBinding<MyKind>> = exact("A").bind(A);
  /// # }
  /// ```
  pub fn bind<NewKind, ViaKind>(
    self,
    kind: ViaKind,
  ) -> Action<TokenKindIdBinding<NewKind>, ActionState, ErrorType>
  where
    ViaKind: SubTokenKind<TokenKind = TokenKindIdBinding<NewKind>>
      + Into<TokenKindIdBinding<NewKind>>
      + Clone
      + 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    macro_rules! impl_bind {
      ($exec: ident) => {
        Box::new(move |input| {
          $exec(input).map(|output| ActionOutput {
            kind: kind.clone().into(),
            digested: output.digested,
            error: output.error,
          })
        })
      };
    }

    Action {
      kind: ViaKind::kind_id(),
      head: self.head,
      muted: self.muted,
      literal: self.literal,
      exec: match self.exec {
        ActionExec::Immutable(exec) => ActionExec::Immutable(impl_bind!(exec)),
        ActionExec::Mutable(exec) => ActionExec::Mutable(impl_bind!(exec)),
      },
    }
  }

  /// Set the kind to the default for this action.
  /// # Examples
  /// ```
  /// use whitehole::lexer::{
  ///   action::{Action, exact},
  ///   token::{TokenKindIdBinding, token_kind},
  /// };
  ///
  /// #[token_kind]
  /// #[derive(Clone, Debug, Default)]
  /// enum MyKind { #[default] Anonymous, A }
  ///
  /// # fn main() {
  /// let action: Action<TokenKindIdBinding<MyKind>> = exact("A").bind_default();
  /// # }
  /// ```
  pub fn bind_default<NewKind>(self) -> Action<TokenKindIdBinding<NewKind>, ActionState, ErrorType>
  where
    NewKind: DefaultTokenKindIdBinding<NewKind>,
    ActionState: 'static,
    ErrorType: 'static,
  {
    macro_rules! impl_bind_default {
      ($exec: ident) => {
        Box::new(move |input| {
          $exec(input).map(|output| ActionOutput {
            kind: TokenKindIdBinding::default(),
            digested: output.digested,
            error: output.error,
          })
        })
      };
    }

    Action {
      kind: NewKind::default_kind_id(),
      head: self.head,
      muted: self.muted,
      literal: self.literal,
      exec: match self.exec {
        ActionExec::Immutable(exec) => ActionExec::Immutable(impl_bind_default!(exec)),
        ActionExec::Mutable(exec) => ActionExec::Mutable(impl_bind_default!(exec)),
      },
    }
  }

  /// Set the kind and the data binding for this action by the selector.
  /// Use this if you need to calculate the kind based on the [`ActionInput`] and [`ActionOutput`].
  /// # Examples
  /// ```
  /// use whitehole::lexer::{
  ///   action::{Action, regex},
  ///   token::{TokenKindIdBinding, token_kind},
  /// };
  ///
  /// #[token_kind]
  /// #[derive(Clone, Debug)]
  /// enum MyKind { Num(i32) }
  ///
  /// # fn main() {
  /// let action: Action<TokenKindIdBinding<MyKind>> = regex(r"^\d+")
  ///   .select(|ctx| Num(ctx.content().parse().unwrap()));
  /// # }
  /// ```
  pub fn select<NewKind, ViaKind>(
    self,
    selector: impl Fn(
        AcceptedActionOutputContext<
          // user can't mutate the input
          &ActionInput<&ActionState>,
          // output is consumed except the error
          ActionOutput<Kind, &Option<ErrorType>>,
        >,
      ) -> ViaKind
      + 'static,
  ) -> Action<TokenKindIdBinding<NewKind>, ActionState, ErrorType>
  where
    ViaKind:
      Into<TokenKindIdBinding<NewKind>> + SubTokenKind<TokenKind = TokenKindIdBinding<NewKind>>,
    ActionState: 'static,
    ErrorType: 'static,
  {
    macro_rules! impl_select {
      ($exec: ident, $to_mutable: ident) => {
        Box::new(move |input| {
          $exec(input).map(|output| {
            ActionOutput {
              kind: selector(AcceptedActionOutputContext {
                input: action_input_to_ref!(input, $to_mutable),
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
        })
      };
    }

    Action {
      kind: ViaKind::kind_id(),
      head: self.head,
      muted: self.muted,
      literal: self.literal,
      exec: match self.exec {
        ActionExec::Immutable(exec) => ActionExec::Immutable(impl_select!(exec, false)),
        ActionExec::Mutable(exec) => ActionExec::Mutable(impl_select!(exec, true)),
      },
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
    assert_eq!(action.kind, A::kind_id());
    assert!(matches!(
      action.exec.as_immutable()(& ActionInput::new("A", 0, & ()).unwrap()),
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
    assert_eq!(action.kind, MyKind::default_kind_id());
    assert!(matches!(
      action.exec.as_immutable()(& ActionInput::new("A", 0, & ()).unwrap()),
      Some(ActionOutput {
        kind: binding,
        digested: 1,
        error: None
      }) if matches!(binding.value(), MyKind::A) && binding.id() == MyKind::default_kind_id()
    ));
  }

  #[test]
  fn action_select() {
    let action: Action<_> =
      Action::from(regex(r"^\d+")).select(|ctx| Value(ctx.content().parse().unwrap()));
    assert_eq!(action.kind, Value::kind_id());
    assert!(matches!(
      action.exec.as_immutable()(& ActionInput::new("1", 0, & ()).unwrap()),
      Some(ActionOutput {
        kind: binding,
        digested: 1,
        error: None
      }) if matches!(binding.value(), MyKind::Value(value) if value.0 == 1) && binding.id() == Value::kind_id()
    ));
  }
}
