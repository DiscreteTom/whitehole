use super::AcceptedActionOutputContext;
use crate::lexer::{
  action::{Action, ActionExec, ActionInput, ActionOutput},
  token::{DefaultTokenKindId, SubTokenKind, TokenKindIdBinding},
};

impl<Kind: 'static, State: 'static, Heap: 'static> Action<Kind, State, Heap> {
  /// Set the binding for this action.
  /// Use this if your action can only yield a const token kind value.
  /// # Examples
  /// ```
  /// use whitehole::lexer::{
  ///   action::{Action, exact},
  ///   token::token_kind,
  /// };
  ///
  /// #[token_kind]
  /// #[derive(Clone, Debug)]
  /// enum MyKind { A, B(i32) }
  ///
  /// # fn main() {
  /// let action: Action<MyKind> = exact("A").bind(A);
  /// let action: Action<MyKind> = exact("A").bind(B(0));
  /// # }
  /// ```
  pub fn bind<NewKind, ViaKind>(self, kind: ViaKind) -> Action<NewKind, State, Heap>
  where
    ViaKind:
      SubTokenKind<TokenKind = NewKind> + Into<TokenKindIdBinding<NewKind>> + Clone + 'static,
  {
    let exec = self.exec.raw;
    Action {
      kind: ViaKind::kind_id(),
      head: self.head,
      muted: self.muted,
      literal: self.literal,
      exec: ActionExec::new(move |input| {
        exec(input).map(|output| ActionOutput {
          binding: kind.clone().into(),
          digested: output.digested,
        })
      }),
    }
  }

  /// Set the kind to the default for this action.
  /// # Examples
  /// ```
  /// use whitehole::lexer::{
  ///   action::{Action, exact},
  ///   token::{token_kind, SubTokenKind},
  /// };
  ///
  /// #[token_kind]
  /// #[derive(Clone, Debug, Default)]
  /// enum MyKind { #[default] Anonymous, A }
  ///
  /// # fn main() {
  /// let action: Action<MyKind> = exact("A").bind_default();
  /// assert_eq!(action.kind(), Anonymous::kind_id());
  /// # }
  /// ```
  pub fn bind_default<NewKind>(self) -> Action<NewKind, State, Heap>
  where
    NewKind: DefaultTokenKindId + Default,
  {
    let exec = self.exec.raw;
    Action {
      kind: NewKind::default_kind_id(),
      head: self.head,
      muted: self.muted,
      literal: self.literal,
      exec: ActionExec::new(move |input| {
        exec(input).map(|output| ActionOutput {
          binding: TokenKindIdBinding::default(),
          digested: output.digested,
        })
      }),
    }
  }

  /// Set the kind and the data binding for this action by the `selector`.
  /// Use this if you need to calculate the kind based on the [`ActionInput`] and [`ActionOutput`].
  ///
  /// [`ActionInput::state`] is immutable in the `selector`.
  /// You can consume the [`ActionOutput::binding`] in the `selector`
  /// but not the [`ActionOutput::error`].
  /// # Examples
  /// ```
  /// use whitehole::lexer::{
  ///   action::{Action, regex},
  ///   token::token_kind,
  /// };
  ///
  /// #[token_kind]
  /// #[derive(Clone, Debug)]
  /// enum MyKind { Num(i32) }
  ///
  /// # fn main() {
  /// let action: Action<MyKind> = regex(r"^\d+")
  ///   .select(|ctx| Num(ctx.content().parse().unwrap()));
  /// # }
  /// ```
  pub fn select<NewKind, ViaKind>(
    self,
    selector: impl Fn(
        AcceptedActionOutputContext<
          &mut ActionInput<&mut State, &mut Heap>,
          ActionOutput<Kind>,
        >,
      ) -> ViaKind
      + 'static,
  ) -> Action<NewKind, State, Heap>
  where
    ViaKind: Into<TokenKindIdBinding<NewKind>> + SubTokenKind<TokenKind = NewKind>,
  {
    let exec = self.exec.raw;
    Action {
      kind: ViaKind::kind_id(),
      head: self.head,
      muted: self.muted,
      literal: self.literal,
      exec: ActionExec::new(move |input| {
        exec(input).map(|output| ActionOutput {
          digested: output.digested,
          binding: selector(AcceptedActionOutputContext { input, output }).into(),
        })
      }),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{exact, regex};
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
      (action.exec.raw)(&mut ActionInput::new("A", 0, &mut (), &mut ()).unwrap()),
      Some(ActionOutput {
        binding,
        digested: 1,
      }) if matches!(binding.kind(), MyKind::A) && binding.id() == A::kind_id()
    ));
  }

  #[test]
  fn action_bind_default() {
    let action: Action<_> = exact("A").bind_default();
    assert_eq!(action.kind, MyKind::default_kind_id());
    assert!(matches!(
      (action.exec.raw)(&mut ActionInput::new("A", 0, &mut (), &mut ()).unwrap()),
      Some(ActionOutput {
        binding,
        digested: 1,
      }) if matches!(binding.kind(), MyKind::A) && binding.id() == MyKind::default_kind_id()
    ));
  }

  #[test]
  fn action_select() {
    let action: Action<_> =
      Action::from(regex(r"^\d+")).select(|ctx| Value(ctx.content().parse().unwrap()));
    assert_eq!(action.kind, Value::kind_id());
    assert!(matches!(
      (action.exec.raw)(&mut ActionInput::new("1", 0, &mut (), &mut ()).unwrap()),
      Some(ActionOutput {
        binding,
        digested: 1,
      }) if matches!(binding.kind(), MyKind::Value(value) if value.0 == 1) && binding.id() == Value::kind_id()
    ));
  }
}
