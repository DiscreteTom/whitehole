/// The output of [`Action::action`](crate::action::Action::action).
///
/// Usually built by [`Input::digest`](crate::action::Input::digest).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output<'text, Value> {
  /// The parsed value.
  pub value: Value,
  /// The rest of the input text.
  pub rest: &'text str,
}

impl<'text, Value> Output<'text, Value> {
  /// Convert [`Self::value`] to a new value.
  #[inline]
  pub fn map<NewValue>(self, f: impl FnOnce(Value) -> NewValue) -> Output<'text, NewValue> {
    Output {
      value: f(self.value),
      rest: self.rest,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn output_map() {
    assert_eq!(
      Output {
        value: 1,
        rest: "123",
      }
      .map(|value| value + 1),
      Output {
        value: 2,
        rest: "123",
      }
    );
  }
}
