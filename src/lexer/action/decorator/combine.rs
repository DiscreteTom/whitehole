use crate::lexer::action::{Action, ActionInput, SubAction};
use std::ops::Add;

// there should NOT be an `Action::or(Action)` to merge actions,
// because if we implement that, we need to merge two actions' head matcher,
// however the head matcher should NOT be modified since some actions may rely on it.
// e.g. suppose we have an action that only accept one character based on the head matcher
// and don't check the `input.rest` in the `exec`, then give the output a data
// `SubAction::new(|_| Some(1)).into().head_in(['a']).data(|_| 1)`
// merge it with `SubAction::new(|_| Some(1)).into().head_in(['b']).data(|_| 2)`
// the head matcher will be merged to `OneOf(['a', 'b'])` and the action will accept both 'a' and 'b'
// but when the `input.rest` starts with 'b', the action will accept it and set data to `1`
// which is incorrect.

// there should NOT be an `Action::and_then(Action)` either,
// because if we implement that, we should ignore the second action's head matcher,
// however the head matcher should NOT be ignored since some actions may rely on it.
// e.g. suppose we have an action that only accept one character based on the head matcher
// and don't check the `input.rest` in the `exec`
// `SubAction::new(|_| Some(1)).into().head_in(['a'])`
// if we use the action as the second action, the head matcher is ignored and
// the action will accept any character, which is incorrect.

// however, since `SubAction` has no head matcher and other fields,
// there could be `Action::and_then(SubAction)`. we use `Add` to implement this.

impl<Kind, ActionState: 'static, ErrorType: 'static> Add<SubAction<ActionState>>
  for Action<Kind, ActionState, ErrorType>
{
  type Output = Self;

  /// Execute a [`SubAction`] after current [`Action`] is accepted.
  /// Current action's attributes (e.g. [`head_matcher`](Self::head_matcher))
  /// and non-[`digested`](super::ActionOutput::digested) fields in the [`ActionOutput`](super::ActionOutput)
  /// (e.g. [`kind`](super::ActionOutput::kind)) will NOT be changed.
  ///
  /// This is usually used to create actions with [`head_matcher`](Self::head_matcher) set safely
  /// by action [`utils`](super::super::utils).
  /// # Caveats
  /// If the current `Action` digest all the rest of the input text,
  /// the next `SubAction` will NOT be executed and the whole action will be rejected.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{exact, chars, ActionInput, Action};
  /// // `exact("0b")` will set the head matcher for us safely
  /// let binary: Action<_> = exact("0b") + chars(|ch| ch == &'0' || ch == &'1');
  /// assert_eq!(binary.exec(&mut ActionInput::new("0b101", 0, ()).unwrap()).unwrap().digested, 5);
  /// ```
  fn add(mut self, rhs: SubAction<ActionState>) -> Self::Output {
    let exec = self.exec;
    self.exec = Box::new(move |input| {
      exec(input).and_then(|mut output| {
        ActionInput::new(input.text(), input.start() + output.digested, input.state).and_then(
          |mut input| {
            rhs.exec(&mut input).map(|another_digested| {
              output.digested += another_digested;
              // other fields in `output` is not changed (e.g. `output.muted`),
              // so we don't need to change other fields of `self` (e.g. `self.maybe_muted`)
              output
            })
          },
        )
      })
    });
    self
  }
}
