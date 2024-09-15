use super::AcceptedActionOutputContext;
use crate::lexer::{
  action::{Action, ActionInput, ActionOutput},
  token::{MockTokenKind, SubTokenKind},
};

impl<'a, Kind: 'a, State: 'a, Heap: 'a> Action<'a, Kind, State, Heap> {
  /// Set the kind to [`MockTokenKind`] and store the data in [`MockTokenKind::data`].
  /// Return a new action.
  ///
  /// You can consume the original [`ActionOutput`] in the `factory`.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{Action, regex};
  /// # let action: Action<_> =
  /// regex(r"^\d+").data(|ctx| ctx.content().parse::<i32>());
  /// ```
  #[inline]
  pub fn data<T>(
    self,
    factory: impl Fn(
        AcceptedActionOutputContext<&mut ActionInput<&mut State, &mut Heap>, ActionOutput<Kind>>,
      ) -> T
      + 'a,
  ) -> Action<'a, MockTokenKind<T>, State, Heap> {
    self.map_exec_new(MockTokenKind::kind_id(), move |exec, input| {
      exec(input).map(|output| ActionOutput {
        digested: output.digested,
        binding: MockTokenKind {
          data: factory(AcceptedActionOutputContext { input, output }),
        }
        .into(),
      })
    })
  }
}

impl<'a, Data: 'a, State: 'a, Heap: 'a> Action<'a, MockTokenKind<Data>, State, Heap> {
  /// Map the data of the kind to another data, stored in [`MockTokenKind::data`].
  /// Return a new action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{Action, simple_with_data};
  /// # let action: Action<_> =
  /// simple_with_data(|_| Some((1, "data"))).map(|data| data.to_string());
  /// ```
  #[inline]
  pub fn map<NewData>(
    self,
    transformer: impl Fn(Data) -> NewData + 'a,
  ) -> Action<'a, MockTokenKind<NewData>, State, Heap> {
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
      (action.exec.raw)(&mut ActionInput::new("A", 0, &mut (), &mut()).unwrap()),
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
      .map(Box::new)
      .prepare(|input| *input.state += 1);
    assert!(matches!(
      (action.exec.raw)(&mut ActionInput::new("A", 0, &mut 123, &mut ()).unwrap()),
      Some(ActionOutput {
        binding,
        digested: 1,
      }) if **binding.kind().data == 1
    ));
  }
}
