use super::{input::ActionInput, sub, Action, ActionOutput, SubAction, SubActionInput};
use crate::lexer::token::{MockTokenKind, SubTokenKind};

/// Accept a function that digests the rest of the input text and returns the number of digested characters.
/// Return `0` if the action is rejected.
/// # Examples
/// ```
/// use whitehole::lexer::action::{SubAction, simple};
/// // accept all rest characters
/// let a: SubAction<()> = simple(|input| input.rest().len());
/// ```
pub fn simple<ActionState>(
  f: impl Fn(&SubActionInput<ActionState>) -> usize + 'static,
) -> SubAction<ActionState> {
  sub(move |input| match f(input) {
    0 => None,
    digested => Some(digested),
  })
}

/// Provide a function that digests the rest of the input text,
/// returns the number of digested characters and the data.
/// `0` is allowed as an accepted number of digested characters.
/// Return `None` if the action is rejected.
///
/// This is useful if you can directly yield the data in the exec,
/// instead of parsing the [`content`](super::AcceptedActionOutputContext::content)
/// later using [`Action::data`].
///
/// It's recommended to set [`Action::head_matcher`] to optimize the lex performance.
/// # Examples
/// ```
/// use whitehole::lexer::token::MockTokenKind;
/// use whitehole::lexer::action::{Action, simple_with_data};
/// // accept all rest characters and parse them into an integer
/// let a: Action<MockTokenKind<i32>> = simple_with_data(|input| Some(input.rest().len(), input.rest().parse().unwrap()));
/// ```
pub fn simple_with_data<ActionState, ErrorType, T, F>(
  f: F,
) -> Action<MockTokenKind<T>, ActionState, ErrorType>
where
  // ActionInput is immutable so we can set `Action::may_mutate_state` to false.
  F: Fn(&ActionInput<ActionState>) -> Option<(usize, T)> + 'static,
{
  Action {
    exec: Box::new(move |input| match f(input) {
      Some((digested, data)) => Some(ActionOutput {
        kind: MockTokenKind::new(data),
        digested,
        error: None,
      }),
      _ => None,
    }),
    kind_id: MockTokenKind::kind_id(),
    head_matcher: None,
    muted: false,
    may_mutate_state: false,
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
      simple(|input| input.text().len()).exec(&SubActionInput::new("123", 0, &mut ()).unwrap()),
      Some(3)
    ));
  }

  #[test]
  fn simple_accept_rest() {
    assert!(matches!(
      simple(|input| input.rest().len()).exec(&SubActionInput::new("123", 1, &mut ()).unwrap()),
      Some(2)
    ));
  }

  #[test]
  fn simple_reject_on_0() {
    assert!(matches!(
      simple(|_| 0).exec(&SubActionInput::new("123", 0, &mut ()).unwrap()),
      None
    ));
  }

  #[test]
  fn simple_option_with_data_accept() {
    let action: Action<MockTokenKind<u32>> =
      simple_with_data(|input| Some((input.text().len(), 123)));
    let output = action.exec(&mut ActionInput::new("123", 0, &mut ()).unwrap());
    assert!(matches!(
      output,
      Some(ActionOutput {
        kind: MockTokenKind { data: 123 },
        digested: 3,
        error: None
      })
    ));
  }

  #[test]
  fn simple_option_with_data_accept_0() {
    let action: Action<MockTokenKind<u32>> = simple_with_data(|_| Some((0, 123)));
    let output = action.exec(&mut ActionInput::new("123", 0, &mut ()).unwrap());
    assert!(matches!(
      output,
      Some(ActionOutput {
        kind: MockTokenKind { data: 123 },
        digested: 0,
        error: None
      })
    ));
  }

  #[test]
  fn simple_option_with_data_reject() {
    let action: Action<MockTokenKind<u32>> = simple_with_data(|_| None);
    let output = action.exec(&mut ActionInput::new("123", 0, &mut ()).unwrap());
    assert!(matches!(output, None));
  }
}
