use super::AcceptedActionOutputContext;
use crate::lexer::{
  action::{Action, ActionExec, ActionInput, ActionOutput},
  token::{MockTokenKind, SubTokenKind},
};

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  /// Set the kind to [`MockTokenKind`] and store the data in [`MockTokenKind::data`].
  /// Return a new action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{Action, simple};
  /// # let action: Action<_> =
  /// simple(|_| 1).data(|ctx| ctx.content().parse::<i32>());
  /// ```
  pub fn data<T>(
    self,
    factory: impl Fn(
        AcceptedActionOutputContext<
          // user can't mutate the input
          &ActionInput<&ActionState>,
          // output is consumed except the error
          ActionOutput<Kind, &Option<ErrorType>>,
        >,
      ) -> T
      + 'static,
  ) -> Action<MockTokenKind<T>, ActionState, ErrorType>
  where
    ActionState: 'static,
    ErrorType: 'static,
  {
    Action {
      exec: match self.exec {
        ActionExec::Immutable(exec) => ActionExec::Immutable(Box::new(move |input| {
          exec(input).map(|output| ActionOutput {
            kind: MockTokenKind {
              data: factory(AcceptedActionOutputContext {
                input,
                // don't consume the error
                output: ActionOutput {
                  kind: output.kind,
                  digested: output.digested,
                  error: &output.error,
                },
              }),
            },
            digested: output.digested,
            error: output.error,
          })
        })),
        ActionExec::Mutable(exec) => ActionExec::Mutable(Box::new(move |input| {
          exec(input).map(|output| ActionOutput {
            kind: MockTokenKind {
              data: factory(AcceptedActionOutputContext {
                input: &input.as_ref(),
                // don't consume the error
                output: ActionOutput {
                  kind: output.kind,
                  digested: output.digested,
                  error: &output.error,
                },
              }),
            },
            digested: output.digested,
            error: output.error,
          })
        })),
      },
      muted: self.muted,
      head: self.head,
      kind: MockTokenKind::kind_id(),
      literal: self.literal,
    }
  }
}

impl<Data, ActionState, ErrorType> Action<MockTokenKind<Data>, ActionState, ErrorType> {
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
  ) -> Action<MockTokenKind<NewData>, ActionState, ErrorType>
  where
    Data: 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    self.data(move |ctx| transformer(ctx.output.kind.data))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::simple_with_data;

  #[test]
  fn action_data() {
    let action: Action<MockTokenKind<Box<usize>>> = simple_with_data(|_| Some((1, Box::new(1))))
      // ensure output.kind can be consumed
      .data(|ctx| ctx.output.kind.data);
    assert!(matches!(
      action.exec.as_immutable()(&mut ActionInput::new("A", 0, &()).unwrap()),
      Some(ActionOutput {
        kind: MockTokenKind { data },
        digested: 1,
        error: None
      }) if *data == 1
    ));
  }

  #[test]
  fn action_map() {
    let action: Action<MockTokenKind<Box<Box<usize>>>> =
      simple_with_data(|_| Some((1, Box::new(1))))
        // ensure data can be consumed in the transformer
        .map(|data| Box::new(data));
    assert!(matches!(
      action.exec.as_immutable()(&mut ActionInput::new("A", 0, &()).unwrap()),
      Some(ActionOutput {
        kind: MockTokenKind { data },
        digested: 1,
        error: None
      }) if **data == 1
    ));
  }
}
