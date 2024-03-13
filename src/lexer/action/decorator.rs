mod combine;
mod data;
mod head;
mod kind;

pub use kind::*;
// these modules have no exportable items for now
// pub use combine::*;
// pub use data::*;
// pub use head::*;

use super::{
  input::ActionInput,
  output::{ActionOutput, EnhancedActionOutput},
  Action,
};

pub struct AcceptedActionDecoratorContext<InputType, OutputType> {
  pub input: InputType,
  pub output: OutputType,
}

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  /// Check the [`ActionInput`] before the action is executed.
  /// Reject the action if the `condition` returns `true`.
  /// Return a new action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # #[derive(Clone, Default)]
  /// # struct MyState {
  /// #   pub reject: bool,
  /// # }
  /// # let mut builder = LexerBuilder::<MyKind, MyState>::default();
  /// builder.define_with(
  ///   MyKind::A,
  ///   regex(r"^\s+").unwrap(),
  ///   |a| a.prevent(|input| input.state.reject)
  /// );
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

  /// Modify `ActionState` before the action is executed.
  /// This will set [`Self::may_mutate_state`] to `true`.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # #[derive(Clone, Default)]
  /// # struct MyState {
  /// #   pub value: i32,
  /// # }
  /// # let mut builder = LexerBuilder::<MyKind, MyState>::default();
  /// builder.define_with(
  ///   MyKind::A,
  ///   regex(r"^\s+").unwrap(),
  ///   |a| a.prepare(|input| input.state.value += 1)
  /// );
  /// ```
  pub fn prepare<F>(mut self, modifier: F) -> Self
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(&mut ActionInput<ActionState>) + 'static,
  {
    let exec = self.exec;
    self.exec = Box::new(move |input| {
      modifier(input);
      exec(input)
    });
    self.may_mutate_state = true;
    self
  }

  /// Apply a decorator to this action.
  /// Usually used to modify the [`ActionOutput`].
  /// For most cases you don't need to use this directly.
  /// Return a new action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define(
  ///   MyKind::A,
  ///   regex(r"^\s+")
  ///     .unwrap()
  ///     .apply(|mut ctx| {
  ///       ctx.output.muted = true;
  ///       ctx.output.into()
  ///     })
  /// );
  /// ```
  // TODO: make this private, don't let user to mutate output directly
  // because it might break the integrity of maybe_muted or may_mutate_state
  pub fn apply<NewErrorType, F>(self, decorator: F) -> Action<Kind, ActionState, NewErrorType>
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(
        AcceptedActionDecoratorContext<
          // user can mutate input.state
          &mut ActionInput<ActionState>,
          // TODO: don't build EnhancedActionOutput?
          // e.g. add a new method `ActionOutput.enhance(input)`
          // so that user can build the EnhancedActionOutput by themselves on demand?
          // will this improve the performance?
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
      may_mutate_state: self.may_mutate_state,
    }
  }

  /// Set [`ActionOutput::muted`] if the action is accepted.
  /// Return a new action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define(
  ///   MyKind::A,
  ///   regex(r"^\s+")
  ///     .unwrap()
  ///     .mute_if(|ctx| ctx.output.rest().len() > 0)
  /// );
  /// ```
  pub fn mute_if<F>(self, condition: F) -> Self
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(
        // user can't mutate the context
        &AcceptedActionDecoratorContext<
          &mut ActionInput<ActionState>,
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
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define(
  ///   MyKind::A,
  ///   regex(r"^\s+")
  ///     .unwrap()
  ///     .mute(true)
  /// );
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
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind, (), &'static str>::default();
  /// builder.define_with(
  ///   MyKind::A,
  ///   regex(r"^\s+").unwrap(),
  ///   |a| a.check(|ctx| {
  ///     if ctx.output.rest().len() > 0 {
  ///       Some("error")
  ///     } else {
  ///       None
  ///     }
  ///   })
  /// );
  /// ```
  pub fn check<NewError, F>(self, condition: F) -> Action<Kind, ActionState, NewError>
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(
        // user can't mutate the context
        &AcceptedActionDecoratorContext<
          &mut ActionInput<ActionState>,
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
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind, (), &'static str>::default();
  /// builder.define_with(
  ///   MyKind::A,
  ///   regex(r"^\s+").unwrap(),
  ///   |a| a.error("error")
  /// );
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
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define(
  ///   MyKind::A,
  ///   regex(r"^\s+")
  ///     .unwrap()
  ///     .reject_if(|ctx| ctx.output.rest().len() > 0)
  /// );
  /// ```
  pub fn reject_if<F>(self, condition: F) -> Self
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(
        // user can't mutate the context
        &AcceptedActionDecoratorContext<
          &mut ActionInput<ActionState>,
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
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # let mut builder = LexerBuilder::<MyKind>::default();
  /// builder.define(
  ///   MyKind::A,
  ///   regex(r"^\s+")
  ///     .unwrap()
  ///     .reject(true)
  /// );
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
  /// You can modify [`ActionInput::state`] in the `cb`.
  /// This will set [`Self::may_mutate_state`] to `true`.
  /// Return a new action.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder};
  /// # use whitehole_macros::TokenKind;
  /// # #[derive(TokenKind, Clone)]
  /// # enum MyKind { A }
  /// # #[derive(Clone, Default)]
  /// # struct MyState {
  /// #   pub value: i32,
  /// # }
  /// # let mut builder = LexerBuilder::<MyKind, MyState>::default();
  /// builder.define_with(
  ///   MyKind::A,
  ///   regex(r"^\s+").unwrap(),
  ///   |a| a.callback(|ctx| ctx.input.state.value += 1)
  /// );
  /// ```
  pub fn callback<F>(mut self, cb: F) -> Self
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(
        AcceptedActionDecoratorContext<
          // user can mutate the input.state
          &mut ActionInput<ActionState>,
          // user can't mutate the output
          &EnhancedActionOutput<Kind, Option<ErrorType>>,
        >,
      ) + 'static,
  {
    let exec = self.exec;
    self.exec = Box::new(move |input| {
      exec(input).and_then(|output| {
        let output = EnhancedActionOutput::new(&input, output);
        cb(AcceptedActionDecoratorContext {
          output: &output,
          input,
        });
        output.into()
      })
    });
    self.may_mutate_state = true;
    self
  }
}

#[cfg(test)]
mod tests {
  use crate::lexer::{
    action::{
      input::ActionInput, output::ActionOutput, simple::simple, ActionInputRestHeadMatcher,
    },
    token::TokenKind,
    Action,
  };
  use whitehole_macros::_TokenKind;

  #[derive(_TokenKind, Clone)]
  enum MyKind {
    A,
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

    // `action.apply` can modify input.state
    let mut state = MyState { value: 0 };
    let action: Action<(), MyState, ()> =
      simple(|input: &mut ActionInput<MyState>| input.rest().len()).apply(|ctx| {
        ctx.input.state.value += 1;
        ctx.output.into()
      });
    action.exec(&mut ActionInput::new("A", 0, &mut state));
    assert_eq!(state.value, 1);
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
    // ensure callback can update the state
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
}
