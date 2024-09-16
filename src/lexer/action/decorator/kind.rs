use super::AcceptedActionOutputContext;
use crate::{
  kind::{DefaultSubKind, KindIdBinding, SubKind},
  lexer::action::{Action, ActionInput, ActionOutput},
};

impl<'a, Kind: 'a, State: 'a, Heap: 'a> Action<'a, Kind, State, Heap> {
  /// Set [`Self::kind`] to a constant sub kind.
  /// Use this if the action can only yield a constant kind value.
  /// # Examples
  /// ```
  /// use whitehole::{
  ///   kind::whitehole_kind,
  ///   lexer::action::{Action, exact},
  /// };
  ///
  /// #[whitehole_kind]
  /// #[derive(Clone, Debug)]
  /// enum MyKind { A, B(i32) }
  ///
  /// # fn main() {
  /// let action: Action<MyKind> = exact("A").bind(A);
  /// let action: Action<MyKind> = exact("A").bind(B(0));
  /// # }
  /// ```
  #[inline]
  pub fn bind<NewKind, Sub>(self, kind: Sub) -> Action<'a, NewKind, State, Heap>
  where
    Sub: SubKind<Kind = NewKind> + Into<KindIdBinding<NewKind>> + Clone + 'a,
  {
    self.map_exec_new(Sub::kind_id(), move |exec, input| {
      exec(input).map(|output| ActionOutput {
        binding: kind.clone().into(),
        digested: output.digested,
      })
    })
  }

  /// Set [`Self::kind`] to the default sub kind.
  /// # Examples
  /// ```
  /// use whitehole::{
  ///   kind::{whitehole_kind, SubKind},
  ///   lexer::action::{Action, exact},
  /// };
  ///
  /// #[whitehole_kind]
  /// #[derive(Clone, Debug, Default)]
  /// enum MyKind { #[default] Anonymous }
  ///
  /// # fn main() {
  /// let action: Action<MyKind> = exact("a").bind_default();
  /// assert_eq!(action.kind(), Anonymous::kind_id());
  /// # }
  /// ```
  #[inline]
  pub fn bind_default<NewKind>(self) -> Action<'a, NewKind, State, Heap>
  where
    NewKind: DefaultSubKind + Default,
  {
    self.map_exec_new(NewKind::default_kind_id(), move |exec, input| {
      exec(input).map(|output| ActionOutput {
        binding: KindIdBinding::default(),
        digested: output.digested,
      })
    })
  }

  /// Set [`Self::kind`] by the `selector`.
  /// Use this if you need to calculate the kind value based on the [`ActionInput`] and [`ActionOutput`].
  ///
  /// You can consume the original [`ActionOutput`] in the `selector`.
  /// # Examples
  /// ```
  /// use whitehole::{kind::whitehole_kind, lexer::action::{Action, regex}};
  ///
  /// #[whitehole_kind]
  /// #[derive(Clone, Debug)]
  /// enum MyKind { Num(i32) }
  ///
  /// # fn main() {
  /// let action: Action<MyKind> = regex(r"^\d+")
  ///   .select(|ctx| Num(ctx.content().parse().unwrap()));
  /// # }
  /// ```
  #[inline]
  pub fn select<NewKind, Sub>(
    self,
    selector: impl Fn(
        AcceptedActionOutputContext<&mut ActionInput<&mut State, &mut Heap>, ActionOutput<Kind>>,
      ) -> Sub
      + 'a,
  ) -> Action<'a, NewKind, State, Heap>
  where
    Sub: Into<KindIdBinding<NewKind>> + SubKind<Kind = NewKind>,
  {
    self.map_exec_new(Sub::kind_id(), move |exec, input| {
      exec(input).map(|output| ActionOutput {
        digested: output.digested,
        binding: selector(AcceptedActionOutputContext { input, output }).into(),
      })
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{exact, regex};
  use whitehole_macros::_whitehole_kind;

  #[_whitehole_kind]
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
