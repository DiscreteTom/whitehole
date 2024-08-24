use super::AcceptedActionOutputContext;
use crate::lexer::{
  action::{Action, ActionExec, ActionInput, ActionOutput},
  token::{MockTokenKind, SubTokenKind, TokenKindIdBinding},
};

impl<Kind: 'static, State: 'static> Action<Kind, State> {
  /// Set the kind to [`MockTokenKind`] and store the data in [`MockTokenKind::data`].
  /// Return a new action.
  ///
  /// You can consume the [`ActionOutput::binding`] in the `factory`
  /// but not the [`ActionOutput::error`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{Action, regex};
  /// # let action: Action<_> =
  /// regex(r"^\d+").data(|ctx| ctx.content().parse::<i32>());
  /// ```
  pub fn data<T>(
    self,
    factory: impl Fn(
        AcceptedActionOutputContext<
          &mut ActionInput<&mut State>,
          ActionOutput<TokenKindIdBinding<Kind>>,
        >,
      ) -> T
      + 'static,
  ) -> Action<MockTokenKind<T>, State> {
    let exec = self.exec.raw;
    Action {
      exec: ActionExec::new(move |input| {
        exec(input).map(|output| ActionOutput {
          digested: output.digested,
          binding: MockTokenKind {
            data: factory(AcceptedActionOutputContext { input, output }),
          }
          .into(),
        })
      }),
      muted: self.muted,
      head: self.head,
      kind: MockTokenKind::kind_id(),
      literal: self.literal,
    }
  }
}

impl<Data: 'static, State: 'static> Action<MockTokenKind<Data>, State> {
  /// Map the data of the kind to another data, stored in [`MockTokenKind::data`].
  /// Return a new action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{Action, simple_with_data};
  /// # let action: Action<_> =
  /// simple_with_data(|_| Some((1, "data"))).map(|data| data.to_string());
  /// ```
  pub fn map<NewData>(
    self,
    transformer: impl Fn(Data) -> NewData + 'static,
  ) -> Action<MockTokenKind<NewData>, State> {
    self.data(move |ctx| transformer(ctx.output.binding.take().data))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::simple_with_data;

  #[test]
  fn action_data() {
    let action: Action<_> = simple_with_data(|_| Some((1, Box::new(1))))
      // ensure output.binding can be consumed
      .data(|ctx| ctx.output.binding.take().data);
    assert!(matches!(
      (action.exec.raw)(&mut ActionInput::new("A", 0, &mut ()).unwrap()),
      Some(ActionOutput {
        binding,
        digested: 1,
      }) if *binding.kind().data == 1
    ));
  }

  #[test]
  fn action_map() {
    let action: Action<_, i32> = simple_with_data(|_| Some((1, Box::new(1))))
      // ensure data can be consumed in the transformer
      .map(|data| Box::new(data))
      .prepare(|input| *input.state += 1);
    assert!(matches!(
      (action.exec.raw)(&mut ActionInput::new("A", 0, &mut 123).unwrap()),
      Some(ActionOutput {
        binding,
        digested: 1,
      }) if **binding.kind().data == 1
    ));
  }
}
