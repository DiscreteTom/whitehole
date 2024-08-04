use super::AcceptedActionOutputContext;
use crate::lexer::{
  action::{map_exec_adapt_input, mut_input_to_ref, Action, ActionExec, ActionInput, ActionOutput},
  token::{MockTokenKind, SubTokenKind, TokenKindIdBinding},
};

impl<Kind, State, ErrorType> Action<Kind, State, ErrorType> {
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
          &ActionInput<&State>,
          // output is consumed except the error
          ActionOutput<TokenKindIdBinding<Kind>, &Option<ErrorType>>,
        >,
      ) -> T
      + 'static,
  ) -> Action<MockTokenKind<T>, State, ErrorType>
  where
    State: 'static,
    ErrorType: 'static,
  {
    macro_rules! impl_data {
      ($exec: ident, $mut_input_to_ref: ident) => {
        Box::new(move |input| {
          $exec(input).map(|output| ActionOutput {
            binding: MockTokenKind {
              data: factory(AcceptedActionOutputContext {
                input: mut_input_to_ref!(input, $mut_input_to_ref),
                // don't consume the error
                output: ActionOutput {
                  binding: output.binding,
                  digested: output.digested,
                  error: &output.error,
                },
              }),
            }
            .into(),
            digested: output.digested,
            error: output.error,
          })
        })
      };
    }

    Action {
      exec: map_exec_adapt_input!(self.exec, impl_data),
      muted: self.muted,
      head: self.head,
      kind: MockTokenKind::kind_id(),
      literal: self.literal,
    }
  }
}

impl<Data, State, ErrorType> Action<MockTokenKind<Data>, State, ErrorType> {
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
  ) -> Action<MockTokenKind<NewData>, State, ErrorType>
  where
    Data: 'static,
    State: 'static,
    ErrorType: 'static,
  {
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
      action.exec.as_immutable()(&mut ActionInput::new("A", 0, &()).unwrap()),
      Some(ActionOutput {
        binding,
        digested: 1,
        error: None
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
      action.exec.as_mutable()(&mut ActionInput::new("A", 0, &mut 123).unwrap()),
      Some(ActionOutput {
        binding,
        digested: 1,
        error: None
      }) if **binding.kind().data == 1
    ));
  }
}
