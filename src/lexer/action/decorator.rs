use super::{
  input::ActionInput,
  output::{ActionOutput, EnhancedActionOutput},
  Action, ActionInputRestHeadMatcher,
};
use crate::lexer::token::{MockTokenKind, TokenKind, TokenKindId};
use std::{collections::HashSet, ops};

/// `input.state` is mutable. `output` is consumed.
pub struct AcceptedActionDecoratorContext<'input, InputType, OutputType> {
  pub input: &'input mut InputType,
  pub output: OutputType,
}

/// `input.state` is mutable. `output` is not mutable and not consumed.
pub struct ActionCallbackContext<'input, 'output, InputType, OutputType> {
  pub input: &'input mut InputType,
  pub output: &'output OutputType,
}

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  /// Set [`Action::head_matcher`] to [`OneOf`](ActionInputRestHeadMatcher::OneOf).
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{Action, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define_with(MyKind::A, |a| {
  ///   a.regex(r"^A")
  ///     .unwrap()
  ///     .head_in(['A'])
  ///     .into()
  /// });
  /// ```
  pub fn head_in(mut self, char_set: impl Into<HashSet<char>>) -> Self {
    self.head_matcher = Some(ActionInputRestHeadMatcher::OneOf(char_set.into()));
    self
  }

  /// Set [`Action::head_matcher`] to [`Not`](ActionInputRestHeadMatcher::Not).
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{Action, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define_with(MyKind::A, |a| {
  ///   a.regex(r"^A")
  ///     .unwrap()
  ///     .head_not(['B'])
  ///     .into()
  /// });
  /// ```
  pub fn head_not(mut self, char_set: impl Into<HashSet<char>>) -> Self {
    self.head_matcher = Some(ActionInputRestHeadMatcher::Not(char_set.into()));
    self
  }

  /// Set [`Action::head_matcher`] to [`Unknown`](ActionInputRestHeadMatcher::Unknown).
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{Action, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define_with(MyKind::A, |a| {
  ///   a.regex(r"^A")
  ///     .unwrap()
  ///     .head_unknown()
  ///     .into()
  /// });
  /// ```
  pub fn head_unknown(mut self) -> Self {
    self.head_matcher = Some(ActionInputRestHeadMatcher::Unknown);
    self
  }

  /// Check the [`ActionInput`] before the action is executed.
  /// Reject the action if the `condition` returns `true`.
  /// Return a new action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{Action, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # #[derive(Clone, Default)]
  /// # struct MyState {
  /// #   pub reject: bool,
  /// # }
  /// # let mut builder = LexerBuilder::<MyKind, MyState>::default();
  /// builder.define_with(MyKind::A, |a| {
  ///   a.regex(r"^\s+")
  ///     .unwrap()
  ///     .prevent(|input| input.state.reject)
  ///     .into()
  /// });
  /// ```
  pub fn prevent<F>(mut self, condition: F) -> Self
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(&ActionInput<ActionState>) -> bool + 'static,
  {
    let exec = self.exec;
    self.exec = Box::new(
      move |input| {
        if condition(input) {
          None
        } else {
          exec(input)
        }
      },
    );
    self
  }

  /// Apply a decorator to this action.
  /// Usually used to modify the [`ActionOutput`].
  /// For most cases you don't need to use this directly.
  /// Return a new action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{Action, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define_with(MyKind::A, |a| {
  ///   a.regex(r"^\s+")
  ///     .unwrap()
  ///     .apply(|ctx| ctx.output.into())
  ///     .into()
  /// });
  /// ```
  pub fn apply<NewErrorType, F>(self, decorator: F) -> Action<Kind, ActionState, NewErrorType>
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(
        AcceptedActionDecoratorContext<
          ActionInput<ActionState>,
          EnhancedActionOutput<Kind, Option<ErrorType>>,
        >,
      ) -> Option<ActionOutput<Kind, Option<NewErrorType>>>
      + 'static,
  {
    let exec = self.exec;
    Action {
      exec: Box::new(move |input| {
        exec(input).and_then(|output| {
          decorator(AcceptedActionDecoratorContext {
            output: EnhancedActionOutput::new(input, output),
            input,
          })
        })
      }),
      maybe_muted: self.maybe_muted,
      possible_kinds: self.possible_kinds,
      head_matcher: self.head_matcher,
    }
  }

  /// Set [`ActionOutput::muted`] if the action is accepted.
  /// Return a new action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{Action, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define_with(MyKind::A, |a| {
  ///   a.regex(r"^\s+")
  ///     .unwrap()
  ///     .mute_if(|ctx| ctx.output.rest().len() > 0)
  ///     .into()
  /// });
  /// ```
  pub fn mute_if<F>(self, condition: F) -> Self
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(
        &AcceptedActionDecoratorContext<
          ActionInput<ActionState>,
          EnhancedActionOutput<Kind, Option<ErrorType>>,
        >,
      ) -> bool
      + 'static,
  {
    let mut res = self.apply(move |mut ctx| {
      ctx.output.muted = condition(&ctx);
      ctx.output.into()
    });
    // we can't know whether the output will be muted
    // so we set maybe_muted to true
    res.maybe_muted = true;
    res
  }

  /// Set [`ActionOutput::muted`] if the action is accepted.
  /// Return a new action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{Action, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define_with(MyKind::A, |a| {
  ///   a.regex(r"^\s+")
  ///     .unwrap()
  ///     .mute(true)
  ///     .into()
  /// });
  /// ```
  pub fn mute(self, muted: bool) -> Self
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    // reminder: DON'T use `self.mute_if(move |_| muted)`
    // because we can set `maybe_muted` to `muted` directly
    let mut res = self.apply(move |mut ctx| {
      ctx.output.muted = muted;
      ctx.output.into()
    });
    res.maybe_muted = muted; // we know this
    res
  }

  /// Set [`ActionOutput::error`] if the action is accepted.
  /// Return a new action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{Action, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind, (), &'static str>::default();
  /// builder.define_with(MyKind::A, |a| {
  ///   a.regex(r"^\s+").unwrap().check(|ctx| {
  ///     if ctx.output.rest().len() > 0 {
  ///       Some("error")
  ///     } else {
  ///       None
  ///     }
  ///   }).into()
  /// });
  /// ```
  pub fn check<NewError, F>(self, condition: F) -> Action<Kind, ActionState, NewError>
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(
        &AcceptedActionDecoratorContext<
          ActionInput<ActionState>,
          EnhancedActionOutput<Kind, Option<ErrorType>>,
        >,
      ) -> Option<NewError>
      + 'static,
  {
    self.apply(move |ctx| {
      Some(ActionOutput {
        error: condition(&ctx),
        kind: ctx.output.base.kind,
        digested: ctx.output.base.digested,
        muted: ctx.output.base.muted,
      })
    })
  }

  /// Set [`ActionOutput::error`] if the action is accepted.
  /// Return a new action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{Action, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind, (), &'static str>::default();
  /// builder.define_with(MyKind::A, |a| {
  ///   a.regex(r"^\s+").unwrap().error("error").into()
  /// });
  /// ```
  pub fn error<NewError>(self, error: NewError) -> Action<Kind, ActionState, NewError>
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    NewError: Clone + 'static,
  {
    self.check(move |_| Some(error.clone()))
  }

  /// Reject the action if the condition is met.
  /// Return a new action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{Action, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define_with(MyKind::A, |a| {
  ///   a.regex(r"^\s+")
  ///     .unwrap()
  ///     .reject_if(|ctx| ctx.output.rest().len() > 0)
  ///     .into()
  /// });
  /// ```
  pub fn reject_if<F>(self, condition: F) -> Self
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(
        &AcceptedActionDecoratorContext<
          ActionInput<ActionState>,
          EnhancedActionOutput<Kind, Option<ErrorType>>,
        >,
      ) -> bool
      + 'static,
  {
    self.apply(move |ctx| {
      if condition(&ctx) {
        None
      } else {
        ctx.output.into()
      }
    })
  }

  /// Reject the action by the given value.
  /// Return a new action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{Action, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define_with(MyKind::A, |a| {
  ///   a.regex(r"^\s+")
  ///     .unwrap()
  ///     .reject(true)
  ///     .into()
  /// });
  /// ```
  pub fn reject(self, rejected: bool) -> Self
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    self.reject_if(move |_| rejected)
  }

  /// Call the `cb` if the action is accepted.
  /// This is often used to update the action state.
  /// Return a new action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{Action, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # #[derive(Clone, Default)]
  /// # struct MyState {
  /// #   pub value: i32,
  /// # }
  /// # let mut builder = LexerBuilder::<MyKind, MyState>::default();
  /// builder.define_with(MyKind::A, |a| {
  ///   a.regex(r"^\s+")
  ///     .unwrap()
  ///     .callback(|ctx| ctx.input.state.value += 1)
  ///     .into()
  /// });
  /// ```
  pub fn callback<F>(mut self, cb: F) -> Self
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(
        ActionCallbackContext<
          ActionInput<ActionState>,
          EnhancedActionOutput<Kind, Option<ErrorType>>,
        >,
      ) + 'static,
  {
    let exec = self.exec;
    self.exec = Box::new(move |input| {
      exec(input).and_then(|output| {
        let output = EnhancedActionOutput::new(&input, output);
        cb(ActionCallbackContext {
          output: &output,
          input,
        });
        output.into()
      })
    });
    self
  }

  /// Execute another action if current action can't be accepted.
  /// Return a new action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{Action, LexerBuilder, action::exact};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define_with(MyKind::A, |a| {
  ///   a.from(exact("A"))
  ///     .or(exact("AA"))
  ///     .into()
  /// });
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
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{Action, LexerBuilder, action::exact};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { AB }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// let mut lexer = builder.define_with(MyKind::AB, |a| {
  ///   a.from(exact("A"))
  ///     .and(exact("B"))
  ///     .into()
  /// }).build("AB");
  /// assert_eq!(lexer.lex().token.unwrap().content, "AB");
  /// // use `+` as a shortcut
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define(MyKind::AB, exact("A") + exact("B"));
  /// ```
  pub fn and<NewKind>(
    self,
    another: Action<NewKind, ActionState, ErrorType>,
  ) -> Action<NewKind, ActionState, ErrorType>
  where
    Kind: 'static,
    NewKind: 'static,
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
      head_matcher: self.head_matcher,
      // `self.maybe_muted` is ignored since only the `output.digested` is used
      maybe_muted: another.maybe_muted,
      // `self.possible_kinds` is ignored since only the `output.digested` is used
      possible_kinds: another.possible_kinds,
    }
  }

  /// Set the kind and the data binding for this action.
  /// Use this if your action can only yield one kind.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{Action, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.append_with(|a| {
  ///   a.regex(r"^\s+")
  ///     .unwrap()
  ///     .bind(MyKind::A)
  ///     .into()
  /// });
  /// ```
  pub fn bind<NewKind>(self, kind: impl Into<NewKind>) -> Action<NewKind, ActionState, ErrorType>
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    NewKind: TokenKind<NewKind> + Clone + 'static,
  {
    let kind = kind.into();
    self.kind_ids([kind.id()]).select(move |_| kind.clone())
  }

  // TODO: add comments and tests
  pub fn map<T, F>(self, transformer: F) -> Action<MockTokenKind<T>, ActionState, ErrorType>
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(Kind) -> T + 'static,
  {
    let exec = self.exec;
    Action {
      exec: Box::new(move |input| {
        exec(input).map(|output| ActionOutput {
          kind: MockTokenKind {
            data: transformer(output.kind),
          },
          digested: output.digested,
          muted: output.muted,
          error: output.error,
        })
      }),
      maybe_muted: self.maybe_muted,
      head_matcher: self.head_matcher,
      possible_kinds: MockTokenKind::possible_kinds(),
    }
    // since there is just on possible kinds in MockTokenKind
    // we don't need to call `action.kinds().select()` here
  }
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static> ops::BitOr<Self>
  for Action<Kind, ActionState, ErrorType>
{
  type Output = Self;

  fn bitor(self, rhs: Self) -> Self::Output {
    self.or(rhs)
  }
}

impl<NewKind: 'static, Kind: 'static, ActionState: 'static, ErrorType: 'static>
  ops::Add<Action<NewKind, ActionState, ErrorType>> for Action<Kind, ActionState, ErrorType>
{
  type Output = Action<NewKind, ActionState, ErrorType>;

  fn add(self, rhs: Action<NewKind, ActionState, ErrorType>) -> Self::Output {
    self.and(rhs)
  }
}

#[cfg(test)]
mod tests {
  use crate::lexer::{
    action::{
      input::ActionInput, output::ActionOutput, regex::regex, simple::simple,
      ActionInputRestHeadMatcher,
    },
    token::TokenKind,
    Action,
  };
  use whitehole_macros::_TokenKind;

  #[derive(_TokenKind, Clone)]
  enum MyKind {
    A,
    B,
  }
  #[derive(Clone, Default)]
  struct MyState {
    pub value: i32,
  }

  #[test]
  fn action_prevent() {
    let mut state = MyState { value: 0 };
    let output = simple::<MyState, (), _>(|input| {
      // update the state if the action is executed
      input.state.value += 1;
      // digest all rest
      input.rest().len()
    })
    // prevent the action if the rest is not empty
    .prevent(|input| input.rest().len() > 0)
    .exec(&mut ActionInput::new(" ", 0, &mut state));
    assert!(matches!(output, None));
    assert_eq!(state.value, 0); // the state is not updated
  }

  #[test]
  fn action_apply() {
    let action: Action<MyKind, (), i32> = simple(|input| input.rest().len())
      .mute(true)
      .bind(MyKind::A)
      .head_in(['A'])
      .apply(|mut ctx| {
        ctx.output.digested = 0;
        ctx.output.error = Some(123);
        ctx.output.into()
      });

    // ensure `action.apply` won't change `maybe_muted`, `possible_kinds`, and `head_matcher`
    assert!(action.maybe_muted);
    assert_eq!(action.possible_kinds.len(), 1);
    assert!(action.possible_kinds.contains(&MyKind::A.id()));
    assert!(matches!(
      &action.head_matcher,
      Some(ActionInputRestHeadMatcher::OneOf(set)) if set.contains(&'A') && set.len() == 1
    ));

    // `action.apply` can modify the output and set error
    assert!(matches!(
      action.exec(&mut ActionInput::new("A", 0, &mut ())),
      Some(ActionOutput {
        kind: MyKind::A,
        digested: 0,
        muted: true,
        error: Some(123)
      })
    ));
  }

  #[test]
  fn action_mute_if() {
    let action: Action<(), (), ()> = simple(|_| 1).mute_if(|ctx| ctx.output.rest().len() > 0);

    // ensure `action.mute_if` will set `maybe_muted` to true
    assert!(action.maybe_muted);

    // `action.mute_if` can mute the output
    assert!(matches!(
      action.exec(&mut ActionInput::new("AA", 0, &mut ())),
      Some(ActionOutput {
        kind: (),
        digested: 1,
        muted: true,
        error: None
      })
    ));
  }

  #[test]
  fn action_mute() {
    let muted_action: Action<(), (), ()> = simple(|_| 1).mute(true);
    let not_muted_action: Action<(), (), ()> = simple(|_| 1).mute(false);

    // ensure `action.mute` will set `maybe_muted`
    assert!(muted_action.maybe_muted);
    assert!(!not_muted_action.maybe_muted);

    assert!(matches!(
      muted_action.exec(&mut ActionInput::new("A", 0, &mut ())),
      Some(ActionOutput {
        kind: (),
        digested: 1,
        muted: true,
        error: None
      })
    ));
    assert!(matches!(
      not_muted_action.exec(&mut ActionInput::new("A", 0, &mut ())),
      Some(ActionOutput {
        kind: (),
        digested: 1,
        muted: false,
        error: None
      })
    ));
  }

  #[test]
  fn action_check() {
    let action = simple::<_, &'static str, _>(|_| 1).check(|ctx| {
      if ctx.output.rest().len() > 0 {
        Some("error")
      } else {
        None
      }
    });

    assert!(matches!(
      action.exec(&mut ActionInput::new("A", 0, &mut ())),
      Some(ActionOutput {
        kind: (),
        digested: 1,
        muted: false,
        error: None
      })
    ));
    assert!(matches!(
      action.exec(&mut ActionInput::new("AA", 0, &mut ())),
      Some(ActionOutput {
        kind: (),
        digested: 1,
        muted: false,
        error: Some("error")
      })
    ));
  }

  #[test]
  fn action_error() {
    let action: Action<(), (), &'static str> = simple::<_, &'static str, _>(|_| 1).error("error");

    assert!(matches!(
      action.exec(&mut ActionInput::new("A", 0, &mut ())),
      Some(ActionOutput {
        kind: (),
        digested: 1,
        muted: false,
        error: Some("error")
      })
    ));
  }

  #[test]
  fn action_reject_if() {
    let action: Action<()> = simple(|_| 1).reject_if(|ctx| ctx.output.rest().len() > 0);

    assert!(matches!(
      action.exec(&mut ActionInput::new("A", 0, &mut ())),
      Some(ActionOutput {
        kind: (),
        digested: 1,
        muted: false,
        error: None
      })
    ));
    assert!(matches!(
      action.exec(&mut ActionInput::new("AA", 0, &mut ())),
      None
    ));
  }

  #[test]
  fn action_reject() {
    let rejected_action: Action<()> = simple(|_| 1).reject(true);
    let not_rejected_action: Action<()> = simple(|_| 1).reject(false);

    assert!(matches!(
      rejected_action.exec(&mut ActionInput::new("A", 0, &mut ())),
      None
    ));
    assert!(matches!(
      not_rejected_action.exec(&mut ActionInput::new("A", 0, &mut ())),
      Some(ActionOutput {
        kind: (),
        digested: 1,
        muted: false,
        error: None
      })
    ));
  }

  #[test]
  fn action_callback() {
    let mut state = MyState { value: 0 };
    let action: Action<(), MyState, ()> =
      simple(|input: &mut ActionInput<MyState>| input.rest().len())
        .callback(|ctx| ctx.input.state.value += 1);

    assert!(matches!(
      action.exec(&mut ActionInput::new("A", 0, &mut state)),
      Some(ActionOutput {
        kind: (),
        digested: 1,
        muted: false,
        error: None
      })
    ));
    assert_eq!(state.value, 1);
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
  fn action_bind() {
    let action: Action<MyKind> = simple(|_| 1).bind(MyKind::A);
    assert_eq!(action.possible_kinds.len(), 1);
    assert!(action.possible_kinds.contains(&MyKind::A.id()));
    assert!(matches!(
      action.exec(&mut ActionInput::new("A", 0, &mut ())),
      Some(ActionOutput {
        kind: MyKind::A,
        digested: 1,
        muted: false,
        error: None
      })
    ));
  }

  #[test]
  fn action_head_in() {
    let action: Action<()> = simple(|_| 1).head_in(['a']);
    assert!(matches!(
      action.head_matcher,
      Some(ActionInputRestHeadMatcher::OneOf(set)) if set.contains(&'a') && set.len() == 1
    ));
  }

  #[test]
  fn action_head_not() {
    let action: Action<(), (), ()> = regex(r"^a").unwrap().head_not(['b']);
    assert!(matches!(
      action.head_matcher,
      Some(ActionInputRestHeadMatcher::Not(set)) if set.contains(&'b') && set.len() == 1
    ));
  }

  #[test]
  fn action_head_unknown() {
    let action: Action<()> = simple(|_| 1).head_unknown();
    assert!(matches!(
      action.head_matcher,
      Some(ActionInputRestHeadMatcher::Unknown)
    ));
  }
}
