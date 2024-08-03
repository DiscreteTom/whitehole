use super::{input::ActionInput, Action, ActionExec, ActionOutput};
use crate::lexer::token::{MockTokenKind, SubTokenKind};

/// Accept a function that digests the rest of the input text and returns the number of digested bytes.
/// You can't modify the action state in this function.
/// The function should return `0` if the action is rejected.
///
/// It's recommended to set [`Action::head`] to optimize the lex performance.
/// # Examples
/// ```
/// use whitehole::lexer::action::{Action, simple};
/// // accept all rest characters
/// let a: Action<_> = simple(|input| input.rest().len());
/// ```
pub fn simple<State, ErrorType>(
  // ActionInput is immutable so we can set `Action::may_mutate_state` to false.
  f: impl Fn(&ActionInput<&State>) -> usize + 'static,
) -> Action<MockTokenKind<()>, State, ErrorType> {
  Action {
    exec: ActionExec::Immutable(Box::new(move |input| match f(input) {
      0 => None,
      digested => Some(ActionOutput {
        binding: MockTokenKind::new(()).into(),
        digested,
        error: None,
      }),
    })),
    kind: MockTokenKind::kind_id(),
    head: None,
    muted: false,
    literal: None,
  }
}

/// Provide a function that digests the rest of the input text and
/// returns the number of digested bytes and the data.
/// `0` is ***allowed*** as an accepted number of digested bytes.
/// Return `None` if the action is rejected.
///
/// This is useful if you can directly yield the data in the function,
/// instead of parsing the [`content`](super::AcceptedActionOutputContext::content)
/// later using [`Action::data`].
///
/// It's recommended to set [`Action::head`] to optimize the lex performance.
/// # Examples
/// ```
/// use whitehole::lexer::token::MockTokenKind;
/// use whitehole::lexer::action::{Action, simple_with_data};
/// // accept all rest characters and parse them into an integer
/// let a: Action<MockTokenKind<i32>> = simple_with_data(|input| Some((input.rest().len(), input.rest().parse().unwrap())));
/// ```
pub fn simple_with_data<State, ErrorType, T>(
  // ActionInput is immutable so we can set `Action::may_mutate_state` to false.
  f: impl Fn(&ActionInput<&State>) -> Option<(usize, T)> + 'static,
) -> Action<MockTokenKind<T>, State, ErrorType> {
  Action {
    exec: ActionExec::Immutable(Box::new(move |input| match f(input) {
      Some((digested, data)) => Some(ActionOutput {
        binding: MockTokenKind::new(data).into(),
        digested,
        error: None,
      }),
      _ => None,
    })),
    kind: MockTokenKind::kind_id(),
    head: None,
    muted: false,
    literal: None,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::output::ActionOutput;

  #[test]
  fn simple_accept_all() {
    assert!(matches!(
      simple::<_, ()>(|input| input.text().len())
        .exec
        .as_immutable()(&ActionInput::new("123", 0, &()).unwrap())
      .unwrap()
      .digested,
      3
    ));
  }

  #[test]
  fn simple_accept_rest() {
    assert!(matches!(
      simple::<_, ()>(|input| input.rest().len())
        .exec
        .as_immutable()(&ActionInput::new("123", 1, &()).unwrap())
      .unwrap()
      .digested,
      2
    ));
  }

  #[test]
  fn simple_reject_on_0() {
    assert!(matches!(
      simple::<_, ()>(|_| 0).exec.as_immutable()(&ActionInput::new("123", 0, &()).unwrap()),
      None
    ));
  }

  #[test]
  fn simple_option_with_data_accept() {
    let action: Action<MockTokenKind<u32>> =
      simple_with_data(|input| Some((input.text().len(), 123)));
    let output = action.exec.as_immutable()(&ActionInput::new("123", 0, &()).unwrap());
    assert!(matches!(
      output,
      Some(ActionOutput {
        binding,
        digested: 3,
        error: None
      }) if binding.kind().data == 123
    ));
  }

  #[test]
  fn simple_option_with_data_accept_0() {
    let action: Action<MockTokenKind<u32>> = simple_with_data(|_| Some((0, 123)));
    let output = action.exec.as_immutable()(&ActionInput::new("123", 0, &()).unwrap());
    assert!(matches!(
      output,
      Some(ActionOutput {
        binding,
        digested: 0,
        error: None
      }) if binding.kind().data == 123
    ));
  }

  #[test]
  fn simple_option_with_data_reject() {
    let action: Action<MockTokenKind<u32>> = simple_with_data(|_| None);
    let output = action.exec.as_immutable()(&ActionInput::new("123", 0, &()).unwrap());
    assert!(matches!(output, None));
  }
}
