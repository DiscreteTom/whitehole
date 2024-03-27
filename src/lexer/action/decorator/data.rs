use super::AcceptedActionDecoratorContext;
use crate::lexer::{
  action::{Action, ActionInput, ActionOutput, EnhancedActionOutput},
  token::{MockTokenKind, SubTokenKind},
};

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  /// Set the kind to [`MockTokenKind`] and store the data in [`MockTokenKind::data`].
  /// Return a new action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{Action, simple};
  /// # let action: Action<_> =
  /// simple(|_| 1).data(|ctx| ctx.output.content().parse::<i32>());
  /// ```
  pub fn data<T, F>(self, factory: F) -> Action<MockTokenKind<T>, ActionState, ErrorType>
  where
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
      ) -> T
      + 'static,
  {
    let exec = self.exec;
    Action {
      exec: Box::new(move |input| {
        exec(input).map(|output| ActionOutput {
          kind: MockTokenKind {
            data: factory(AcceptedActionDecoratorContext {
              // don't consume the error
              output: ActionOutput {
                kind: output.kind,
                digested: output.digested,
                muted: output.muted,
                error: &output.error,
              }
              .into_enhanced(input),
              input,
            }),
          },
          digested: output.digested,
          muted: output.muted,
          error: output.error,
        })
      }),
      may_mutate_state: self.may_mutate_state,
      maybe_muted: self.maybe_muted,
      head_matcher: self.head_matcher,
      kind_id: MockTokenKind::kind_id(),
    }
  }
}

impl<Data, ActionState, ErrorType> Action<MockTokenKind<Data>, ActionState, ErrorType> {
  /// Map the data of the kind to another data, stored in [`MockTokenKind::data`].
  /// Return a new action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{Action, simple_option_with_data};
  /// # let action: Action<_> =
  /// simple_option_with_data(|_| Some((1, "data"))).map(|data| data.to_string());
  /// ```
  pub fn map<NewData, F>(
    self,
    transformer: F,
  ) -> Action<MockTokenKind<NewData>, ActionState, ErrorType>
  where
    Data: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(Data) -> NewData + 'static,
  {
    let exec = self.exec;
    Action {
      exec: Box::new(move |input| {
        exec(input).map(|output| ActionOutput {
          kind: MockTokenKind {
            data: transformer(output.kind.data),
          },
          digested: output.digested,
          muted: output.muted,
          error: output.error,
        })
      }),
      may_mutate_state: self.may_mutate_state,
      maybe_muted: self.maybe_muted,
      head_matcher: self.head_matcher,
      kind_id: MockTokenKind::kind_id(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::simple_option_with_data;

  #[test]
  fn action_data() {
    let action: Action<MockTokenKind<Box<usize>>> =
      simple_option_with_data(|_| Some((1, Box::new(1))))
        // ensure output.kind can be consumed
        .data(|ctx| ctx.output.base.kind.data);
    assert!(matches!(
      action.exec(&mut ActionInput::new("A", 0, ())),
      Some(ActionOutput {
        kind: MockTokenKind { data },
        digested: 1,
        muted: false,
        error: None
      }) if *data == 1
    ));
  }

  #[test]
  fn action_map() {
    let action: Action<MockTokenKind<Box<Box<usize>>>> =
      simple_option_with_data(|_| Some((1, Box::new(1))))
        // ensure data can be consumed in the transformer
        .map(|data| Box::new(data));
    assert!(matches!(
      action.exec(&mut ActionInput::new("A", 0, ())),
      Some(ActionOutput {
        kind: MockTokenKind { data },
        digested: 1,
        muted: false,
        error: None
      }) if **data == 1
    ));
  }
}
