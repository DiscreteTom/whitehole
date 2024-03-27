use super::{Action, ActionInput, ActionOutput};
use crate::lexer::token::{MockTokenKind, SubTokenKind};
use std::ops::{Add, BitOr};

/// A light weight version of [`Action`].
/// [`Self::exec`] only returns the number of characters digested if the action is accepted,
/// and return [`None`] if the action is rejected.
/// [`ActionInput::state`] is NOT mutable in [`Self::exec`].
///
/// [`SubAction`]s can be combined with `|` (shorthand for [`Self::or`])
/// and `+` (shorthand for [`Self::and_then`]) to create new `SubAction`s.
///
/// Low-level action utils like [`chars`](super::chars) will create [`SubAction`]
/// instead of [`Action`] to avoid unnecessary overhead. These utils
/// are usually used to be combined with other `SubAction`s to create more complex actions.
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
pub struct SubAction<ActionState> {
  // make sure the `ActionInput` is not mutable
  // so we can set `Action::may_mutate_state` to `false`
  // when transforming `SubAction` into `Action`.
  exec: Box<dyn Fn(&ActionInput<ActionState>) -> Option<usize>>,
}

impl<ActionState> SubAction<ActionState> {
  /// See [`SubAction`].
  pub fn new(exec: impl Fn(&ActionInput<ActionState>) -> Option<usize> + 'static) -> Self {
    Self {
      exec: Box::new(exec),
    }
  }

  pub fn exec(&self, input: &ActionInput<ActionState>) -> Option<usize> {
    (self.exec)(input)
  }

  /// Execute another [`SubAction`] if current `SubAction` can't be accepted.
  /// You can use `|` as a shorthand. Return a new `SubAction`.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{chars, ActionInput, SubAction};
  /// let ab: SubAction<()> = chars(|ch| ch == &'a') | chars(|ch| ch == &'b');
  /// assert!(matches!(ab.exec(&ActionInput::new("b", 0, ())), Some(1)));
  /// ```
  pub fn or(self, another: impl Into<Self>) -> Self
  where
    ActionState: 'static,
  {
    let exec = self.exec;
    let another_exec = another.into().exec;
    Self {
      exec: Box::new(move |input| exec(input).or_else(|| another_exec(input))),
    }
  }

  /// Execute another [`SubAction`] after the current `SubAction` is accepted.
  /// You can use `+` as a shorthand. Return a new `SubAction`.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{chars, ActionInput, SubAction};
  /// let ab: SubAction<()> = chars(|ch| ch == &'a') + chars(|ch| ch == &'b');
  /// assert!(matches!(ab.exec(&ActionInput::new("ab", 0, ())), Some(2)));
  /// ```
  pub fn and_then(self, another: impl Into<Self>) -> Self
  where
    // in real cases the `ActionState` is a reference type so it is Copy
    ActionState: Copy + 'static,
  {
    let exec = self.exec;
    let another_exec = another.into().exec;
    Self {
      exec: Box::new(move |input| {
        exec(input).and_then(|digested| {
          another_exec(&ActionInput::new(
            input.text(),
            input.start() + digested,
            input.state,
          ))
          .map(|another_digested| digested + another_digested)
        })
      }),
    }
  }
}

impl<ActionState: 'static> BitOr<Self> for SubAction<ActionState> {
  type Output = Self;

  fn bitor(self, rhs: Self) -> Self::Output {
    self.or(rhs)
  }
}

impl<ActionState: Copy + 'static> Add<Self> for SubAction<ActionState> {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    self.and_then(rhs)
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
      // SubAction never mutate the action state
      may_mutate_state: false,
    }
  }
}

// this should NEVER be implemented because the `Action::exec` may mutate the action state
// and break the integrity of `Action::may_mutate_state`.
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
    let a = SubAction::new(|input| match input.rest().len() {
      0 => None,
      digested => Some(digested),
    });

    // accept
    assert_eq!(a.exec(&ActionInput::new("123", 0, ())), Some(3));
    assert_eq!(a.exec(&ActionInput::new("123", 1, ())), Some(2));

    // reject
    assert_eq!(a.exec(&ActionInput::new("", 0, ())), None);
    assert_eq!(a.exec(&ActionInput::new("123", 3, ())), None);
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
      action.exec(&mut ActionInput::new("123", 0, ())),
      Some(ActionOutput {
        kind: mock,
        digested: 3,
        muted: false,
        error: None,
      }) if matches!(mock.data, ())
    ));

    // reject
    assert!(matches!(
      action.exec(&mut ActionInput::new("", 0, ())),
      None
    ));
  }

  #[test]
  fn sub_action_or() {
    // first sub action is accepted
    assert!(matches!(
      SubAction::new(|_| Some(1))
        .or(SubAction::new(|_| Some(2)))
        .exec(&ActionInput::new("123", 0, ())),
      Some(1)
    ));
    assert!(matches!(
      (SubAction::new(|_| Some(1)) | SubAction::new(|_| Some(2))).exec(&ActionInput::new(
        "123",
        0,
        ()
      )),
      Some(1)
    ));

    // first sub action is rejected but the second is accepted
    assert!(matches!(
      SubAction::new(|_| None)
        .or(SubAction::new(|_| Some(2)))
        .exec(&ActionInput::new("123", 0, ())),
      Some(2)
    ));
    assert!(matches!(
      (SubAction::new(|_| None) | SubAction::new(|_| Some(2))).exec(&ActionInput::new(
        "123",
        0,
        ()
      )),
      Some(2)
    ));

    // both sub actions are rejected
    assert_eq!(
      SubAction::new(|_| None)
        .or(SubAction::new(|_| None))
        .exec(&ActionInput::new("123", 0, ())),
      None
    );
    assert_eq!(
      (SubAction::new(|_| None) | SubAction::new(|_| None)).exec(&ActionInput::new("123", 0, ())),
      None
    );
  }

  #[test]
  fn sub_action_and_then() {
    // both sub actions are accepted
    assert!(matches!(
      SubAction::new(|_| Some(1))
        .and_then(SubAction::new(|_| Some(2)))
        .exec(&ActionInput::new("123", 0, ())),
      Some(3)
    ));
    assert!(matches!(
      (SubAction::new(|_| Some(1)) + SubAction::new(|_| Some(2))).exec(&ActionInput::new(
        "123",
        0,
        ()
      )),
      Some(3)
    ));

    // first sub action is accepted but the second is rejected
    assert_eq!(
      SubAction::new(|_| Some(1))
        .and_then(SubAction::new(|_| None))
        .exec(&ActionInput::new("123", 0, ())),
      None
    );
    assert_eq!(
      (SubAction::new(|_| Some(1)) + SubAction::new(|_| None)).exec(&ActionInput::new(
        "123",
        0,
        ()
      )),
      None
    );

    // first sub action is rejected
    assert_eq!(
      SubAction::new(|_| None)
        .and_then(SubAction::new(|_| Some(2)))
        .exec(&ActionInput::new("123", 0, ())),
      None
    );
    assert_eq!(
      (SubAction::new(|_| None) + SubAction::new(|_| Some(2))).exec(&ActionInput::new(
        "123",
        0,
        ()
      )),
      None
    );
  }
}
