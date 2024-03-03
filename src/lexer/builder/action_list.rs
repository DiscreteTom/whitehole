use crate::lexer::Action;

/// A helper struct to accept one or more actions.
pub struct ActionList<ActionType>(pub Vec<ActionType>);

impl<Kind, ActionState, ErrorType> From<Action<Kind, ActionState, ErrorType>>
  for ActionList<Action<Kind, ActionState, ErrorType>>
{
  fn from(value: Action<Kind, ActionState, ErrorType>) -> Self {
    Self(vec![value])
  }
}
impl<Kind, ActionState, ErrorType> From<Vec<Action<Kind, ActionState, ErrorType>>>
  for ActionList<Action<Kind, ActionState, ErrorType>>
{
  fn from(value: Vec<Action<Kind, ActionState, ErrorType>>) -> Self {
    Self(value)
  }
}
impl<Kind, ActionState, ErrorType, const N: usize> From<[Action<Kind, ActionState, ErrorType>; N]>
  for ActionList<Action<Kind, ActionState, ErrorType>>
{
  fn from(value: [Action<Kind, ActionState, ErrorType>; N]) -> Self {
    Self(value.into())
  }
}
