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
