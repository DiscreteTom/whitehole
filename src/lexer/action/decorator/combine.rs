use crate::lexer::{
  action::{ActionInput, ActionOutput, EnhancedActionOutput},
  token::{MockTokenKind, TokenKind},
  Action,
};
use std::ops::{Add, BitOr};

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  /// Execute another action if current action can't be accepted.
  /// Return a new action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{Action, LexerBuilder, action::exact};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define(MyKind::A, exact("A").or(exact("AA")));
  /// // use `|` as a shortcut
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define(MyKind::A, exact("A") | exact("AA"));
  /// ```
  pub fn or(mut self, another: Self) -> Self
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    let exec = self.exec;
    let another_exec = another.exec;
    self.exec = Box::new(move |input| exec(input).or_else(|| another_exec(input)));
    self.maybe_muted = self.maybe_muted || another.maybe_muted;
    self.possible_kinds.extend(another.possible_kinds); // merge possible kinds
    self
  }

  /// Execute another action after the current action is accepted.
  /// Current action's [`maybe_muted`](Self::maybe_muted), [`possible_kinds`](Self::possible_kinds)
  /// and generated [`kind`](ActionOutput::kind), [`muted`](ActionOutput::muted),
  /// [`error`](ActionOutput::error) are ignored.
  /// Next action's [`head_matcher`](Self::head_matcher) is ignored.
  /// Return a new action.
  ///
  /// If you want to retrieve the output of the first action, see [`Self::and_then`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{Action, LexerBuilder, action::exact};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { AB }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// let mut lexer = builder.define(
  ///   MyKind::AB, exact("A").and(exact("B"))
  /// ).build("AB");
  /// assert_eq!(lexer.lex().token.unwrap().content, "AB");
  ///
  /// // use `+` as a shortcut
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define(MyKind::AB, exact("A") + exact("B"));
  /// ```
  pub fn and<NewKind>(
    self,
    another: Action<NewKind, ActionState, ErrorType>,
  ) -> Action<NewKind, ActionState, ErrorType>
  where
    NewKind: 'static,
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    let exec = self.exec;
    let another_exec = another.exec;
    Action {
      exec: Box::new(move |input| {
        exec(input).and_then(|o1| {
          // if the first action is accepted, exec the second action
          another_exec(&mut ActionInput::new(
            input.text(),
            input.start() + o1.digested,
            input.state,
          ))
          .map(|mut o2| {
            // merge the digested length
            o2.digested += o1.digested;
            o2
          })
        })
      }),
      may_mutate_state: self.may_mutate_state || another.may_mutate_state,
      head_matcher: self.head_matcher,
      // `self.maybe_muted` is ignored since only the `output.digested` is used
      maybe_muted: another.maybe_muted,
      // `self.possible_kinds` is ignored since only the `output.digested` is used
      possible_kinds: another.possible_kinds,
    }
  }

  /// Execute another action after the current action is accepted.
  /// Current action's [`maybe_muted`](Self::maybe_muted) and [`possible_kinds`](Self::possible_kinds)
  /// are ignored. Next action's [`head_matcher`](Self::head_matcher) is ignored.
  /// Return a new action with [`MockTokenKind`] as the kind.
  /// You can retrieve the output of both actions and store them in [`MockTokenKind::data`]
  /// for further processing.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::action::{Action, exact};
  /// # let action: Action<_> =
  /// exact("A").and_then(exact("B"), |o1, o2| (o1.base.kind, o2.base.kind));
  /// ```
  pub fn and_then<AnotherKind, ResultType, F>(
    self,
    another: Action<AnotherKind, ActionState, ErrorType>,
    data_factory: F,
  ) -> Action<MockTokenKind<ResultType>, ActionState, ErrorType>
  where
    AnotherKind: 'static,
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(
        EnhancedActionOutput<Kind, Option<ErrorType>>,
        EnhancedActionOutput<AnotherKind, &Option<ErrorType>>,
      ) -> ResultType
      + 'static,
  {
    let exec = self.exec;
    let another_exec = another.exec;
    Action {
      exec: Box::new(move |input_1| {
        exec(input_1).and_then(|output_1| {
          // the first action is accepted, exec the second action
          let text = input_1.text();
          let input_1_start = input_1.start();
          let mut input_2 =
            ActionInput::new(text, input_1_start + output_1.digested, input_1.state);
          another_exec(&mut input_2).map(|output_2| {
            ActionOutput {
              digested: output_1.digested + output_2.digested,
              kind: MockTokenKind {
                data: data_factory(
                  // consume the output_1
                  EnhancedActionOutput {
                    base: output_1,
                    text,
                    start: input_1_start,
                  },
                  // don't consume the output_2 because we need the error
                  // but we can consume output_2.kind
                  // so we build a new ActionOutput
                  EnhancedActionOutput {
                    base: ActionOutput {
                      kind: output_2.kind,
                      digested: output_2.digested,
                      muted: output_2.muted,
                      error: &output_2.error,
                    },
                    text,
                    start: input_2.start(),
                  },
                ),
              },
              muted: output_2.muted,
              error: output_2.error,
            }
          })
        })
      }),
      may_mutate_state: self.may_mutate_state || another.may_mutate_state,
      head_matcher: self.head_matcher,
      // `self.maybe_muted` is ignored because we apply the output_2.muted
      maybe_muted: another.maybe_muted,
      possible_kinds: MockTokenKind::possible_kinds(),
    }
  }
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static> BitOr<Self>
  for Action<Kind, ActionState, ErrorType>
{
  type Output = Self;

  fn bitor(self, rhs: Self) -> Self::Output {
    self.or(rhs)
  }
}

impl<NewKind: 'static, Kind: 'static, ActionState: 'static, ErrorType: 'static>
  Add<Action<NewKind, ActionState, ErrorType>> for Action<Kind, ActionState, ErrorType>
{
  type Output = Action<NewKind, ActionState, ErrorType>;

  fn add(self, rhs: Action<NewKind, ActionState, ErrorType>) -> Self::Output {
    self.and(rhs)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{regex, simple, ActionInputRestHeadMatcher};
  use whitehole_macros::_TokenKind;

  #[derive(_TokenKind, Clone)]
  enum MyKind {
    A,
    B,
  }

  #[test]
  fn action_or() {
    let action: Action<(), (), ()> = regex(r"^a").unwrap().or(regex(r"^b").unwrap());

    assert!(matches!(
      action.exec(&mut ActionInput::new("a", 0, &mut ())),
      Some(ActionOutput {
        kind: (),
        digested: 1,
        muted: false,
        error: None
      })
    ));
    assert!(matches!(
      action.exec(&mut ActionInput::new("b", 0, &mut ())),
      Some(ActionOutput {
        kind: (),
        digested: 1,
        muted: false,
        error: None
      })
    ));

    // use `|` to combine actions
    let action: Action<(), (), ()> = regex(r"^a").unwrap() | regex(r"^b").unwrap();

    assert!(matches!(
      action.exec(&mut ActionInput::new("a", 0, &mut ())),
      Some(ActionOutput {
        kind: (),
        digested: 1,
        muted: false,
        error: None
      })
    ));
    assert!(matches!(
      action.exec(&mut ActionInput::new("b", 0, &mut ())),
      Some(ActionOutput {
        kind: (),
        digested: 1,
        muted: false,
        error: None
      })
    ));

    // maybe_muted should be true if any of the actions is muted
    let action: Action<(), (), ()> = regex(r"^a").unwrap().mute(true) | regex(r"^b").unwrap();
    assert!(action.maybe_muted);
    let action: Action<(), (), ()> = regex(r"^a").unwrap() | regex(r"^b").unwrap().mute(true);
    assert!(action.maybe_muted);
    let action: Action<(), (), ()> =
      regex(r"^a").unwrap().mute(true) | regex(r"^b").unwrap().mute(true);
    assert!(action.maybe_muted);
    let action: Action<(), (), ()> = regex(r"^a").unwrap() | regex(r"^b").unwrap();
    assert!(!action.maybe_muted);

    // possible kinds should be merged
    let action: Action<MyKind, (), ()> =
      regex(r"^a").unwrap().bind(MyKind::A) | regex(r"^b").unwrap().bind(MyKind::B);
    assert_eq!(action.possible_kinds.len(), 2);
    assert!(action.possible_kinds.contains(&MyKind::A.id()));
    assert!(action.possible_kinds.contains(&MyKind::B.id()));
  }

  #[test]
  fn action_and() {
    // reject if any action reject
    let action: Action<(), (), ()> = simple(|_| 0) + simple(|_| 1);
    assert!(matches!(
      action.exec(&mut ActionInput::new("a", 0, &mut ())),
      None
    ));
    let action: Action<(), (), ()> = simple(|_| 1) + simple(|_| 0);
    assert!(matches!(
      action.exec(&mut ActionInput::new("a", 0, &mut ())),
      None
    ));

    let action: Action<(), (), ()> = regex(r"^a").unwrap().and(regex(r"^b").unwrap());
    assert!(matches!(
      action.exec(&mut ActionInput::new("ab", 0, &mut ())),
      Some(ActionOutput {
        kind: (),
        digested: 2,
        muted: false,
        error: None
      })
    ));

    // maybe_muted should be true if the next action is muted
    let action: Action<(), (), ()> = regex(r"^a").unwrap().and(regex(r"^b").unwrap().mute(true));
    assert!(action.maybe_muted);
    // maybe_muted for the first action is ignored
    let action: Action<(), (), ()> = regex(r"^a").unwrap().mute(true).and(regex(r"^b").unwrap());
    assert!(!action.maybe_muted);

    // first action's possible kinds should be ignored
    let action: Action<MyKind, (), ()> = regex(r"^a")
      .unwrap()
      .bind::<MyKind>(MyKind::A)
      .and(regex(r"^b").unwrap().bind(MyKind::B));
    assert_eq!(action.possible_kinds.len(), 1);
    assert!(action.possible_kinds.contains(&MyKind::B.id()));

    // first action's error should be ignored
    let action: Action<(), (), &'static str> = regex::<(), &'static str>(r"^a")
      .unwrap()
      .error("error")
      .and(regex(r"^b").unwrap());
    assert!(matches!(
      action.exec(&mut ActionInput::new("ab", 0, &mut ())),
      Some(ActionOutput {
        kind: (),
        digested: 2,
        muted: false,
        error: None
      })
    ));

    // first action's head matcher is applied
    let action: Action<(), (), ()> = regex(r"^a")
      .unwrap()
      .head_in(['a'])
      .and(regex(r"^b").unwrap().head_in(['b']));
    assert!(matches!(
      action.head_matcher.as_ref().unwrap(),
      ActionInputRestHeadMatcher::OneOf(set) if set.contains(&'a') && set.len() == 1
    ));

    // use '+' as a shortcut
    let action: Action<(), (), ()> = regex(r"^a").unwrap() + regex(r"^b").unwrap();
    assert!(matches!(
      action.exec(&mut ActionInput::new("ab", 0, &mut ())),
      Some(ActionOutput {
        kind: (),
        digested: 2,
        muted: false,
        error: None
      })
    ));
  }

  #[test]
  fn action_and_then() {
    // reject if any action reject
    let action: Action<_, (), ()> = simple(|_| 0).and_then(simple(|_| 1), |_, _| 1);
    assert!(matches!(
      action.exec(&mut ActionInput::new("a", 0, &mut ())),
      None
    ));
    let action: Action<_, (), ()> = simple(|_| 1).and_then(simple(|_| 0), |_, _| 1);
    assert!(matches!(
      action.exec(&mut ActionInput::new("a", 0, &mut ())),
      None
    ));

    // ensure the first output can be consumed, and the second output's kind can be consumed
    let action: Action<_, (), &str> = Action::with_data(|_| {
      Some(ActionOutput {
        kind: MockTokenKind {
          data: Box::new(111),
        },
        digested: 1,
        muted: true,
        error: Some("err1"),
      })
    })
    .and_then(
      Action::with_data(|_| {
        Some(ActionOutput {
          kind: MockTokenKind {
            data: Box::new(222),
          },
          digested: 2,
          muted: false,
          error: Some("err2"),
        })
      }),
      |o1, o2| {
        (
          o1.base,
          o1.start,
          o2.base.kind,
          o2.base.digested,
          o2.base.muted,
          o2.start,
        )
      },
    );
    assert!(matches!(
      action.exec(&mut ActionInput::new("ab", 0, &mut ())),
      Some(ActionOutput {
        kind: MockTokenKind {
          data: (
            // output 1
            ActionOutput {
              // output 1
              kind: MockTokenKind { data: data1 },
              digested: 1,
              muted: true,
              error: Some("err1")
            },
            // input_1.start
            0,
            // output 2's kind, digested and muted
            MockTokenKind { data: data2 },
            2,
            false,
            // input_2.start
            1,
          )
        },
        // the total digested length is 3
        digested: 3,
        // apply second output's muted & error
        muted: false,
        error: Some("err2")
      }) if *data1 == 111 && *data2 == 222
    ));
  }
}
