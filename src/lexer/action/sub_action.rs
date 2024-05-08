use super::{Action, ActionInput, ActionOutput};
use crate::lexer::token::{MockTokenKind, SubTokenKind};
use std::ops::{Add, BitOr};

// TODO: remove this when `ActionInput`'s state is not a reference
/// This has the same interface with [`ActionInput`], but the [`state`](Self::state) is NOT mutable.
pub struct SubActionInput<'text, 'action_state, ActionState> {
  pub state: &'action_state ActionState,
  /// See [`Self::text`].
  text: &'text str,
  /// See [`Self::start`].
  start: usize,
  // cache the rest of the text to prevent create the slice every time
  // because `input.rest` might be used many times
  // when we use `|` to combine sub actions using the same input
  /// See [`Self::rest`].
  rest: &'text str,
}

impl<'text, 'action_state, ActionState> SubActionInput<'text, 'action_state, ActionState> {
  /// Return [`None`] if the [`start`](Self::start) position is out of the input
  /// [`text`](Self::text) or there is no [`rest`](Self::rest).
  pub fn new(text: &'text str, start: usize, state: &'action_state ActionState) -> Option<Self> {
    if start >= text.len() {
      None
    } else {
      Some(Self {
        state,
        text,
        start,
        rest: &text[start..],
      })
    }
  }

  /// Create a new [`SubActionInput`] from an [`ActionInput`].
  pub fn from<'input: 'action_state>(
    input: &'input ActionInput<'text, 'action_state, ActionState>,
  ) -> Self {
    // since `ActionInput` already checks the boundary, we don't need to check it again
    Self {
      state: input.state,
      text: input.text(),
      start: input.start(),
      rest: input.rest(),
    }
  }

  /// From where to lex, in bytes.
  pub fn start(&self) -> usize {
    self.start
  }
  /// The whole input text.
  pub fn text(&self) -> &'text str {
    self.text
  }
  /// The undigested part of the input text.
  /// When lexing this is guaranteed to be not empty.
  pub fn rest(&self) -> &'text str {
    &self.rest
  }
}

/// A light weight version of [`Action`].
/// [`Self::exec`] only returns the number of bytes digested if the action is accepted,
/// and returns [`None`] if the action is rejected.
/// [`ActionInput::state`] is NOT mutable in [`Self::exec`].
///
/// [`SubAction`]s can be combined with [`|`](BitOr) and [`+`](Add)
/// to create new `SubAction`s.
///
/// Low-level action utils like [`simple`](super::simple) will create [`SubAction`]
/// instead of [`Action`] to avoid unnecessary overhead. These utils
/// are usually used to be combined with other `SubAction`s to create more complex actions.
///
/// Use `SubAction::into` to transform `SubAction` into `Action`.
/// # Examples
/// ```
/// # use whitehole::lexer::action::{SubAction, Action, sub};
/// // use `sub` or `SubAction::new` to create a `SubAction`,
/// // accept all rest characters, reject if the rest is empty
/// # let a: SubAction<()> =
/// sub(|input| match input.rest().len() {
///   0 => None,
///   digested => Some(digested),
/// });
/// // transform `SubAction` into `Action`
/// let a: Action<_> = a.into();
/// ```
pub struct SubAction<ActionState> {
  exec: Box<dyn Fn(&SubActionInput<ActionState>) -> Option<usize>>,
}

impl<ActionState> SubAction<ActionState> {
  /// See [`SubAction`]. You can also use [`sub`] as a shortcut.
  pub fn new(exec: impl Fn(&SubActionInput<ActionState>) -> Option<usize> + 'static) -> Self {
    Self {
      exec: Box::new(exec),
    }
  }

  pub fn exec(&self, input: &SubActionInput<ActionState>) -> Option<usize> {
    (self.exec)(input)
  }

  /// Same as [`SubAction::into`] but might be helpful in chaining calls to auto infer the type.
  pub fn into_action<ErrorType>(self) -> Action<MockTokenKind<()>, ActionState, ErrorType>
  where
    ActionState: 'static,
  {
    self.into()
  }
}

impl<ActionState: 'static> BitOr<Self> for SubAction<ActionState> {
  type Output = Self;

  /// Execute another [`SubAction`] if current `SubAction` can't be accepted.
  /// Returns a new `SubAction`.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{chars, ActionInput, SubAction};
  /// let ab: SubAction<()> = chars(|ch| ch == &'a') | chars(|ch| ch == &'b');
  /// assert!(matches!(ab.exec(&SubActionInput::new("b", 0, &mut ())), Some(1)));
  /// ```
  fn bitor(self, rhs: Self) -> Self::Output {
    let exec = self.exec;
    let another_exec = rhs.exec;
    Self {
      exec: Box::new(move |input| exec(input).or_else(|| another_exec(input))),
    }
  }
}

impl<ActionState: 'static> Add<Self> for SubAction<ActionState> {
  type Output = Self;

  /// Execute another [`SubAction`] after the current `SubAction` is accepted.
  /// Returns a new `SubAction`.
  /// # Caveats
  /// If the current `SubAction` digest all the rest of the input text,
  /// the next `SubAction` will NOT be executed and the whole action will be rejected.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{chars, ActionInput, SubAction};
  /// let ab: SubAction<()> = chars(|ch| ch == &'a') + chars(|ch| ch == &'b');
  /// assert!(matches!(ab.exec(&SubActionInput::new("ab", 0, &mut ())), Some(2)));
  /// ```
  fn add(self, rhs: Self) -> Self::Output {
    let exec = self.exec;
    let another_exec = rhs.exec;
    Self {
      exec: Box::new(move |input| {
        exec(input).and_then(|digested| {
          SubActionInput::new(input.text(), input.start() + digested, input.state).and_then(
            |input| another_exec(&input).map(|another_digested| digested + another_digested),
          )
        })
      }),
    }
  }
}

impl<ActionState: 'static, ErrorType> From<SubAction<ActionState>>
  for Action<MockTokenKind<()>, ActionState, ErrorType>
{
  fn from(sub: SubAction<ActionState>) -> Self {
    let exec = sub.exec;
    Self {
      exec: Box::new(move |input| {
        exec(&SubActionInput::from(input)).map(|digested| ActionOutput {
          kind: MockTokenKind::new(()),
          digested,
          error: None,
        })
      }),
      kind_id: MockTokenKind::kind_id(),
      head_matcher: None,
      muted: false,
      // SubAction never mutate the action state
      may_mutate_state: false,
      literal: None,
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

/// Shortcut for [`SubAction::new`].
pub fn sub<ActionState>(
  exec: impl Fn(&SubActionInput<ActionState>) -> Option<usize> + 'static,
) -> SubAction<ActionState> {
  SubAction::new(exec)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn sub_action_exec() {
    let a = SubAction::new(|input| match input.rest().len() {
      1 => None,
      digested => Some(digested),
    });

    // accept
    assert_eq!(
      a.exec(&SubActionInput::new("123", 0, &mut ()).unwrap()),
      Some(3)
    );
    assert_eq!(
      a.exec(&SubActionInput::new("123", 1, &mut ()).unwrap()),
      Some(2)
    );

    // reject
    assert_eq!(
      a.exec(&SubActionInput::new("123", 2, &mut ()).unwrap()),
      None
    );
  }

  #[test]
  fn sub_action_into_action() {
    let action: Action<_> = SubAction::new(|input| match input.rest().len() {
      1 => None,
      digested => Some(digested),
    })
    .into();

    // accept
    assert!(matches!(
      action.exec(&mut ActionInput::new("123", 0, &mut ()).unwrap()),
      Some(ActionOutput {
        kind: mock,
        digested: 3,
        error: None,
      }) if matches!(mock.data, ())
    ));

    // reject
    assert!(matches!(
      action.exec(&mut ActionInput::new("123", 2, &mut ()).unwrap()),
      None
    ));
  }

  #[test]
  fn sub_action_bit_or() {
    // first sub action is accepted
    assert!(matches!(
      (SubAction::new(|_| Some(1)) | SubAction::new(|_| Some(2)))
        .exec(&SubActionInput::new("123", 0, &mut ()).unwrap()),
      Some(1)
    ));

    // first sub action is rejected but the second is accepted
    assert!(matches!(
      (SubAction::new(|_| None) | SubAction::new(|_| Some(2)))
        .exec(&SubActionInput::new("123", 0, &mut ()).unwrap()),
      Some(2)
    ));

    // both sub actions are rejected
    assert_eq!(
      (SubAction::new(|_| None) | SubAction::new(|_| None))
        .exec(&SubActionInput::new("123", 0, &mut ()).unwrap()),
      None
    );
  }

  #[test]
  fn sub_action_add() {
    // both sub actions are accepted
    assert!(matches!(
      (SubAction::new(|_| Some(1)) + SubAction::new(|_| Some(2)))
        .exec(&SubActionInput::new("123", 0, &mut ()).unwrap()),
      Some(3)
    ));

    // first sub action is accepted but the second is rejected
    assert_eq!(
      (SubAction::new(|_| Some(1)) + SubAction::new(|_| None))
        .exec(&SubActionInput::new("123", 0, &mut ()).unwrap()),
      None
    );

    // first sub action is rejected
    assert_eq!(
      (SubAction::new(|_| None) + SubAction::new(|_| Some(2)))
        .exec(&SubActionInput::new("123", 0, &mut ()).unwrap()),
      None
    );

    // caveats: the first sub action digests all the rest
    assert_eq!(
      (SubAction::new(|input| Some(input.rest().len())) + SubAction::new(|_| Some(2)))
        .exec(&SubActionInput::new("123", 0, &mut ()).unwrap()),
      None
    );
  }
}
