use super::{input::ActionInput, sub, Action, ActionOutput, SubAction};
use crate::lexer::token::{MockTokenKind, SubTokenKind};

/// Accept a function that digests the rest of the input text and returns the number of digested characters.
/// Return `0` if the action is rejected.
/// # Examples
/// ```
/// use whitehole::lexer::action::{SubAction, simple};
/// // accept all rest characters, reject if the rest is empty
/// let a: SubAction<()> = simple(|input| input.rest().len());
/// ```
pub fn simple<ActionState>(
  f: impl Fn(&ActionInput<ActionState>) -> usize + 'static,
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
/// It's recommended to set [`Action::head_matcher`] to optimize the lex performance.
/// # Examples
/// ```
/// use whitehole::lexer::action::{Action, simple_option_with_data};
/// // accept all rest characters, never reject.
/// // be ware this may cause infinite loop
/// let a: Action<MockTokenKind<i32>> = simple_option_with_data(|input| Some(input.rest().len(), input.rest().parse().unwrap()));
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
        // make sure the output is never muted
        // so we can set `Action::maybe_muted` to false
        muted: false,
        error: None,
      }),
      _ => None,
    }),
    kind_id: MockTokenKind::kind_id(),
    head_matcher: None,
    maybe_muted: false,
    may_mutate_state: false,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::{
    action::output::ActionOutput,
    token::{TokenKindId, TokenKindIdProvider},
  };

  #[test]
  fn simple_accept_all() {
    let action: Action<MockTokenKind<()>> = simple(|input| input.text().len());
    let output = action.exec(&mut ActionInput::new("123", 0, &mut ()));

    assert!(matches!(
      output,
      Some(ActionOutput {
        kind: mock,
        digested: 3,
        muted: false,
        error: None
      }) if matches!(mock.data, ()) && mock.id() == &TokenKindId::new(0)
    ));
  }

  #[test]
  fn simple_accept_rest() {
    let action: Action<MockTokenKind<()>> = simple(|input| input.rest().len());
    let output = action.exec(&mut ActionInput::new("123", 1, &mut ()));
    assert!(matches!(
      output,
      Some(ActionOutput {
        kind: mock,
        digested: 2,
        muted: false,
        error: None
      }) if matches!(mock.data, ()) && mock.id() == &TokenKindId::new(0)
    ));
  }

  #[test]
  fn simple_reject_on_0() {
    let action: Action<MockTokenKind<()>> = simple(|_| 0);
    let output = action.exec(&mut ActionInput::new("123", 0, &mut ()));
    assert!(matches!(output, None));
  }

  #[test]
  fn simple_option_accept() {
    let action: Action<MockTokenKind<()>> = simple_option(|input| Some(input.text().len()));
    let output = action.exec(&mut ActionInput::new("123", 0, &mut ()));
    assert!(matches!(
      output,
      Some(ActionOutput {
        kind: mock,
        digested: 3,
        muted: false,
        error: None
      }) if matches!(mock.data, ()) && mock.id() == &TokenKindId::new(0)
    ));
  }

  #[test]
  fn simple_option_accept_0() {
    let action: Action<MockTokenKind<()>> = simple_option(|_| Some(0));
    let output = action.exec(&mut ActionInput::new("123", 0, &mut ()));
    assert!(matches!(
      output,
      Some(ActionOutput {
        kind: mock,
        digested: 0,
        muted: false,
        error: None
      }) if matches!(mock.data, ()) && mock.id() == &TokenKindId::new(0)
    ));
  }

  #[test]
  fn simple_option_reject() {
    let action: Action<MockTokenKind<()>> = simple_option(|_| None);
    let output = action.exec(&mut ActionInput::new("123", 0, &mut ()));
    assert!(matches!(output, None));
  }

  #[test]
  fn simple_option_with_data_accept() {
    let action: Action<MockTokenKind<u32>> =
      simple_option_with_data(|input| Some((input.text().len(), 123)));
    let output = action.exec(&mut ActionInput::new("123", 0, &mut ()));
    assert!(matches!(
      output,
      Some(ActionOutput {
        kind: MockTokenKind { data: 123 },
        digested: 3,
        muted: false,
        error: None
      })
    ));
  }

  #[test]
  fn simple_option_with_data_accept_0() {
    let action: Action<MockTokenKind<u32>> = simple_option_with_data(|_| Some((0, 123)));
    let output = action.exec(&mut ActionInput::new("123", 0, &mut ()));
    assert!(matches!(
      output,
      Some(ActionOutput {
        kind: MockTokenKind { data: 123 },
        digested: 0,
        muted: false,
        error: None
      })
    ));
  }

  #[test]
  fn simple_option_with_data_reject() {
    let action: Action<MockTokenKind<u32>> = simple_option_with_data(|_| None);
    let output = action.exec(&mut ActionInput::new("123", 0, &mut ()));
    assert!(matches!(output, None));
  }
}
