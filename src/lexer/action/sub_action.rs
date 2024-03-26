use super::{Action, ActionInput, ActionOutput};
use crate::lexer::token::{MockTokenKind, SubTokenKind};

/// A light weight version of [`Action`].
/// [`Self::exec`] only returns the number of characters digested if the action is accepted,
/// and return [`None`] if the action is rejected.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{SubAction};
/// // accept all rest characters, reject if the rest is empty
/// # let a: SubAction<()> =
/// SubAction::new(|input| match input.rest().len() {
///   0 => None,
///   digested => Some(digested),
/// });
/// ```
pub struct SubAction<ActionState = ()> {
  exec: Box<dyn Fn(&mut ActionInput<ActionState>) -> Option<usize>>,
}

impl<ActionState> SubAction<ActionState> {
  /// See [`SubAction`].
  pub fn new(exec: impl Fn(&mut ActionInput<ActionState>) -> Option<usize> + 'static) -> Self {
    Self {
      exec: Box::new(exec),
    }
  }

  pub fn exec(&self, input: &mut ActionInput<ActionState>) -> Option<usize> {
    (self.exec)(input)
  }
}

impl<ActionState: 'static, ErrorType> Into<Action<MockTokenKind<()>, ActionState, ErrorType>>
  for SubAction<ActionState>
{
  fn into(self) -> Action<MockTokenKind<()>, ActionState, ErrorType> {
    let exec = self.exec;
    Action {
      exec: Box::new(move |input| {
        exec(input).map(|digested| ActionOutput {
          kind: MockTokenKind::new(()),
          digested,
          // make sure the output is never muted
          // so we can set `Action::maybe_muted` to false
          muted: false,
          error: None,
        })
      }),
      kind_id: MockTokenKind::kind_id(),
      head_matcher: None,
      maybe_muted: false,
      may_mutate_state: false,
    }
  }
}

// TODO: is this needed? uncomment when a use case is found
// impl<Kind, ActionState: 'static, ErrorType: 'static> From<Action<Kind, ActionState, ErrorType>>
//   for SubAction<ActionState>
// {
//   fn from(value: Action<Kind, ActionState, ErrorType>) -> Self {
//     let exec = value.exec;
//     Self {
//       exec: Box::new(move |input| exec(input).map(|output| output.digested)),
//     }
//   }
// }

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn sub_action_exec() {
    let a: SubAction<()> = SubAction::new(|input| match input.rest().len() {
      0 => None,
      digested => Some(digested),
    });

    // accept
    assert_eq!(a.exec(&mut ActionInput::new("123", 0, &mut ())), Some(3));
    assert_eq!(a.exec(&mut ActionInput::new("123", 1, &mut ())), Some(2));

    // reject
    assert_eq!(a.exec(&mut ActionInput::new("", 0, &mut ())), None);
    assert_eq!(a.exec(&mut ActionInput::new("123", 3, &mut ())), None);
  }

  #[test]
  fn sub_action_into_action() {
    let action: Action<_> = SubAction::new(|input| match input.rest().len() {
      0 => None,
      digested => Some(digested),
    })
    .into();

    // accept
    assert!(matches!(
      action.exec(&mut ActionInput::new("123", 0, &mut ())),
      Some(ActionOutput {
        kind: mock,
        digested: 3,
        muted: false,
        error: None,
      }) if matches!(mock.data, ())
    ));

    // reject
    assert!(matches!(
      action.exec(&mut ActionInput::new("", 0, &mut ())),
      None
    ));
  }
}
