use crate::lexer::{
  action::{ActionInput, ActionOutput, EnhancedActionOutput},
  token::{MockTokenKind, SubTokenKind},
  Action,
};
use std::ops::{Add, BitOr};

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  /// Execute another action if current action can't be accepted.
  /// The other action's [`head_matcher`](Action::head_matcher) is ignored.
  /// Return a new action.
  /// # Panics
  /// Panics if the kind_id of the two actions are different.
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
    // kind_id must be the same.
    // don't use `assert!` here since it require `Kind: Debug`
    if self.kind_id != another.kind_id {
      panic!(
        "kind_id must be the same, but got {:?} and {:?}",
        self.kind_id.0, another.kind_id.0
      );
    }

    let exec = self.exec;
    let another_exec = another.exec;
    self.exec = Box::new(move |input| exec(input).or_else(|| another_exec(input)));
    // kind_id and head_matcher are not changed
    self.maybe_muted = self.maybe_muted || another.maybe_muted;
    self.may_mutate_state = self.may_mutate_state || another.may_mutate_state;
    self
  }

  /// Execute another action after the current action is accepted.
  /// Current action's [`maybe_muted`](Self::maybe_muted), [`kind_id`](Self::kind_id)
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
      // `self.kind_id` is ignored since only the `output.digested` is used
      kind_id: another.kind_id,
      // next action's head_matcher is ignored
      head_matcher: self.head_matcher,
      // `self.maybe_muted` is ignored since only the `output.digested` is used
      maybe_muted: another.maybe_muted,
      // merge the two actions' `may_mutate_state`
      may_mutate_state: self.may_mutate_state || another.may_mutate_state,
    }
  }

  /// Execute another action after the current action is accepted.
  /// Current action's [`maybe_muted`](Self::maybe_muted), [`kind_id`](Self::kind_id)
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
                      error: &output_2.error, // use ref to avoid consuming the error
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
      kind_id: MockTokenKind::kind_id(),
      head_matcher: self.head_matcher,
      // `self.maybe_muted` is ignored because we apply the output_2.muted
      maybe_muted: another.maybe_muted,
      may_mutate_state: self.may_mutate_state || another.may_mutate_state,
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
  use crate::lexer::{
    action::{regex, simple, HeadMatcher},
    token::TokenKindIdBinding,
  };
  use simple::simple_option_with_data;
  use whitehole_macros::_TokenKind;

  #[derive(_TokenKind, Clone, Debug)]
  enum MyKind {
    A,
    B,
  }

  #[test]
  fn action_or() {
    let action: Action<_> = regex(r"^a").unwrap().or(regex(r"^b").unwrap());

    assert!(matches!(
      action.exec(&mut ActionInput::new("a", 0, &mut ())),
      Some(ActionOutput {
        kind: _,
        digested: 1,
        muted: false,
        error: None
      })
    ));
    assert!(matches!(
      action.exec(&mut ActionInput::new("b", 0, &mut ())),
      Some(ActionOutput {
        kind: _,
        digested: 1,
        muted: false,
        error: None
      })
    ));

    // use `|` to combine actions
    let action: Action<_> = regex(r"^a").unwrap() | regex(r"^b").unwrap();

    assert!(matches!(
      action.exec(&mut ActionInput::new("a", 0, &mut ())),
      Some(ActionOutput {
        kind: _,
        digested: 1,
        muted: false,
        error: None
      })
    ));
    assert!(matches!(
      action.exec(&mut ActionInput::new("b", 0, &mut ())),
      Some(ActionOutput {
        kind: _,
        digested: 1,
        muted: false,
        error: None
      })
    ));

    // maybe_muted should be true if any of the actions is muted
    let action: Action<_> = regex(r"^a").unwrap().mute() | regex(r"^b").unwrap();
    assert!(action.maybe_muted);
    let action: Action<_> = regex(r"^a").unwrap() | regex(r"^b").unwrap().mute();
    assert!(action.maybe_muted);
    let action: Action<_> = regex(r"^a").unwrap().mute() | regex(r"^b").unwrap().mute();
    assert!(action.maybe_muted);
    let action: Action<_> = regex(r"^a").unwrap() | regex(r"^b").unwrap();
    assert!(!action.maybe_muted);

    // possible kinds should be merged
    let action: Action<TokenKindIdBinding<MyKind>> =
      regex(r"^a").unwrap().bind(A) | regex(r"^b").unwrap().bind(B);
    assert_eq!(action.kind_id, A::kind_id());
  }

  #[test]
  fn action_and() {
    // reject if any action reject
    let action: Action<_> = simple(|_| 0) + simple(|_| 1);
    assert!(matches!(
      action.exec(&mut ActionInput::new("a", 0, &mut ())),
      None
    ));
    let action: Action<_> = simple(|_| 1) + simple(|_| 0);
    assert!(matches!(
      action.exec(&mut ActionInput::new("a", 0, &mut ())),
      None
    ));

    let action: Action<_> = regex(r"^a").unwrap().and(regex(r"^b").unwrap());
    assert!(matches!(
      action.exec(&mut ActionInput::new("ab", 0, &mut ())),
      Some(ActionOutput {
        kind: _,
        digested: 2,
        muted: false,
        error: None
      })
    ));

    // maybe_muted should be true if the next action is muted
    let action: Action<_> = regex(r"^a").unwrap().and(regex(r"^b").unwrap().mute());
    assert!(action.maybe_muted);
    // maybe_muted for the first action is ignored
    let action: Action<_> = regex(r"^a").unwrap().mute().and(regex(r"^b").unwrap());
    assert!(!action.maybe_muted);

    // first action's possible kinds should be ignored
    let action: Action<TokenKindIdBinding<MyKind>, (), ()> = regex(r"^a")
      .unwrap()
      .bind(A)
      .and(regex(r"^b").unwrap().bind(B));
    assert_eq!(action.kind_id, B::kind_id());

    // first action's error should be ignored
    let action: Action<_, (), &'static str> = regex::<_, &'static str>(r"^a")
      .unwrap()
      .error("error")
      .and(regex(r"^b").unwrap());
    assert!(matches!(
      action.exec(&mut ActionInput::new("ab", 0, &mut ())),
      Some(ActionOutput {
        kind: _,
        digested: 2,
        muted: false,
        error: None
      })
    ));

    // first action's head matcher is applied
    let action: Action<_> = regex(r"^a")
      .unwrap()
      .unchecked_head_in(['a'])
      .and(regex(r"^b").unwrap().unchecked_head_in(['b']));
    assert!(matches!(
      action.head_matcher.as_ref().unwrap(),
      HeadMatcher::OneOf(set) if set.contains(&'a') && set.len() == 1
    ));

    // use '+' as a shortcut
    let action: Action<_> = regex(r"^a").unwrap() + regex(r"^b").unwrap();
    assert!(matches!(
      action.exec(&mut ActionInput::new("ab", 0, &mut ())),
      Some(ActionOutput {
        kind: _,
        digested: 2,
        muted: false,
        error: None
      })
    ));
  }

  #[test]
  fn action_and_then() {
    // reject if any action reject
    let action: Action<_> = simple(|_| 0).and_then(simple(|_| 1), |_, _| 1);
    assert!(matches!(
      action.exec(&mut ActionInput::new("a", 0, &mut ())),
      None
    ));
    let action: Action<_> = simple(|_| 1).and_then(simple(|_| 0), |_, _| 1);
    assert!(matches!(
      action.exec(&mut ActionInput::new("a", 0, &mut ())),
      None
    ));

    // ensure the first output can be consumed, and the second output's kind can be consumed
    let action: Action<_, (), &str> =
      simple_option_with_data::<(), &str, _, _>(|_| Some((1, Box::new(111))))
        .mute()
        .error("err1")
        .and_then(
          simple_option_with_data::<(), &str, _, _>(|_| Some((2, Box::new(222)))).error("err2"),
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
