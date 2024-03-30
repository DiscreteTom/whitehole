use crate::lexer::action::Action;

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  // there should NOT be an `Action::or` to merge actions,
  // because if we implement that, we need to merge two actions' head matcher,
  // however the head matcher should NOT be modified since some actions may rely on it.
  // e.g. suppose we have an action that only accept one character based on the head matcher
  // and don't check the `input.rest` in the `exec`, then give the output a data
  // `SubAction::new(|_| Some(1)).into().head_in(['a']).data(|_| 1)`
  // merge it with `SubAction::new(|_| Some(1)).into().head_in(['b']).data(|_| 2)`
  // the head matcher will be merged to `OneOf(['a', 'b'])` and the action will accept both 'a' and 'b'
  // but when the `input.rest` starts with 'b', the action will accept it and set data to `1`
  // which is incorrect.

  // there should NOT be an `Action::and_then` either,
  // because if we implement that, we should ignore the second action's head matcher,
  // however the head matcher should NOT be ignored since some actions may rely on it.
  // e.g. suppose we have an action that only accept one character based on the head matcher
  // and don't check the `input.rest` in the `exec`
  // `SubAction::new(|_| Some(1)).into().head_in(['a'])`
  // if we use the action as the second action, the head matcher is ignored and
  // the action will accept any character, which is incorrect.
}
