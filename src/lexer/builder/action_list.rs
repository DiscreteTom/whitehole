use crate::lexer::{
  action::{Action, SubAction},
  token::MockTokenKind,
};

/// A helper struct to accept one or more actions (or sub actions).
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

impl<ActionState: 'static, ErrorType> From<SubAction<ActionState>>
  for ActionList<Action<MockTokenKind<()>, ActionState, ErrorType>>
{
  fn from(value: SubAction<ActionState>) -> Self {
    Self(vec![value.into()])
  }
}
impl<ActionState: 'static, ErrorType> From<Vec<SubAction<ActionState>>>
  for ActionList<Action<MockTokenKind<()>, ActionState, ErrorType>>
{
  fn from(value: Vec<SubAction<ActionState>>) -> Self {
    Self(value.into_iter().map(Into::into).collect())
  }
}
impl<ActionState: 'static, ErrorType, const N: usize> From<[SubAction<ActionState>; N]>
  for ActionList<Action<MockTokenKind<()>, ActionState, ErrorType>>
{
  fn from(value: [SubAction<ActionState>; N]) -> Self {
    Self(value.into_iter().map(Into::into).collect())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::word;

  #[test]
  fn action_list_from_action() {
    let action: Action<_> = word("A");
    let list: ActionList<_> = action.into();
    assert_eq!(list.0.len(), 1);
  }

  #[test]
  fn action_list_from_vec() {
    let action: Action<_> = word("A");
    let list: ActionList<_> = vec![action].into();
    assert_eq!(list.0.len(), 1);
  }

  #[test]
  fn action_list_from_array() {
    let action: Action<_> = word("A");
    let list: ActionList<_> = [action].into();
    assert_eq!(list.0.len(), 1);
  }

  #[test]
  fn action_list_from_sub_action() {
    let action: SubAction<()> = SubAction::new(|_| Some(0));
    let list: ActionList<Action<_>> = action.into();
    assert_eq!(list.0.len(), 1);
  }

  #[test]
  fn action_list_from_sub_action_vec() {
    let action: SubAction<()> = SubAction::new(|_| Some(0));
    let list: ActionList<Action<_>> = vec![action].into();
    assert_eq!(list.0.len(), 1);
  }

  #[test]
  fn action_list_from_sub_action_array() {
    let action: SubAction<()> = SubAction::new(|_| Some(0));
    let list: ActionList<Action<_>> = [action].into();
    assert_eq!(list.0.len(), 1);
  }
}
