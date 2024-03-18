mod combine;
mod data;
mod head;
mod kind;

// these modules have no exportable items for now
// pub use combine::*;
// pub use data::*;
// pub use head::*;
// pub use kind::*;

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

  /// Apply a decorator to this action's output.
  /// Users should NOT use this directly,
  /// because this might break the integrity of [`Action::maybe_muted`] or [`Action::may_mutate_state`].
  /// Return a new action.
  fn apply<NewErrorType, F>(self, decorator: F) -> Action<Kind, ActionState, NewErrorType>
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(
        AcceptedActionDecoratorContext<
          // action state is immutable, we only want to modify the output
          &ActionInput<ActionState>,
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
      kind_id: self.kind_id,
      head_matcher: self.head_matcher,
      may_mutate_state: self.may_mutate_state,
    }
  }

  /// Set [`ActionOutput::muted`] if the action is accepted.
  /// This will set [`Self::maybe_muted`] to `true`.
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
        // user should NOT mutate the output directly
        &AcceptedActionDecoratorContext<
          &ActionInput<ActionState>,
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
    // so we set `maybe_muted` to true
    res.maybe_muted = true;
    res
  }

  /// Set [`ActionOutput::muted`] to `true` if the action is accepted.
  /// This will set [`Self::maybe_muted`] to `true`.
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
  ///     .mute()
  /// );
  /// ```
  pub fn mute(self) -> Self
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    // `mute_if` will set `maybe_muted` to true
    self.mute_if(move |_| true)
  }

  /// Set [`ActionOutput::muted`] to `false` if the action is accepted.
  /// This will set [`Self::maybe_muted`] to `false`.
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
  ///     .unmute()
  /// );
  /// ```
  // this function is needed because this is the only way to set `maybe_muted` to `false`
  // after the construction of Action
  pub fn unmute(self) -> Self
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    let mut res = self.mute_if(move |_| false);
    res.maybe_muted = false;
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
        // user should NOT mutate the output directly
        &AcceptedActionDecoratorContext<
          &ActionInput<ActionState>,
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
        // user should NOT mutate the output directly
        &AcceptedActionDecoratorContext<
          &ActionInput<ActionState>,
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

  /// Reject the action.
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
  ///     .reject()
  /// );
  /// ```
  pub fn reject(self) -> Self
  where
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    self.reject_if(move |_| true)
  }
  // `reject_if(move |_| false)` is meaningless

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
          // user should NOT mutate the output directly
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
    action::{input::ActionInput, output::ActionOutput, simple::simple},
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
    let action = simple::<MyState, (), _>(|input| input.rest().len())
      // modify the state before the action is executed
      .prepare(|input| input.state.value += 1)
      // prevent the action if the rest is empty
      .prevent(|input| input.rest().len() == 0);

    // the first exec, state will be changed, digest all chars
    let output = action.exec(&mut ActionInput::new(" ", 0, &mut state));
    assert!(matches!(output, Some(ActionOutput { digested: 1, .. })));
    assert_eq!(state.value, 1);

    // the second exec, the action is prevented, so the state is not updated
    let output = action.exec(&mut ActionInput::new(" ", 1, &mut state));
    assert!(matches!(output, None));
    assert_eq!(state.value, 0); // the state is not updated
  }

  #[test]
  fn action_prepare() {
    let mut state = MyState { value: 0 };
    let action = simple::<MyState, (), _>(|_| 0)
      // modify the state before the action is executed
      .prepare(|input| input.state.value += 1);

    // ensure may_mutate_state is set to true
    assert!(action.may_mutate_state);

    // the action is rejected, but the state is still updated
    let output = action.exec(&mut ActionInput::new(" ", 0, &mut state));
    assert!(matches!(output, None));
    assert_eq!(state.value, 1);
  }

  #[test]
  fn action_mute_if() {
    let action: Action<_, (), ()> = simple(|_| 1).mute_if(|ctx| ctx.output.rest().len() > 0);

    // ensure `action.mute_if` will set `maybe_muted` to true
    assert!(action.maybe_muted);

    // `action.mute_if` can mute the output
    assert!(matches!(
      action.exec(&mut ActionInput::new("AA", 0, &mut ())),
      Some(ActionOutput { muted: true, .. })
    ));
  }

  #[test]
  fn action_mute_unmute() {
    let muted_action: Action<_, (), ()> = simple(|_| 1).mute();
    let not_muted_action: Action<_, (), ()> = simple(|_| 1).mute().unmute();

    // ensure `action.mute/unmute` will set `maybe_muted`
    assert!(muted_action.maybe_muted);
    assert!(!not_muted_action.maybe_muted);

    assert!(matches!(
      muted_action.exec(&mut ActionInput::new("A", 0, &mut ())),
      Some(ActionOutput { muted: true, .. })
    ));
    assert!(matches!(
      not_muted_action.exec(&mut ActionInput::new("A", 0, &mut ())),
      Some(ActionOutput { muted: false, .. })
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
      Some(ActionOutput { error: None, .. })
    ));
    assert!(matches!(
      action.exec(&mut ActionInput::new("AA", 0, &mut ())),
      Some(ActionOutput {
        error: Some("error"),
        ..
      })
    ));
  }

  #[test]
  fn action_error() {
    let action: Action<_, (), &'static str> = simple::<_, &'static str, _>(|_| 1).error("error");

    assert!(matches!(
      action.exec(&mut ActionInput::new("A", 0, &mut ())),
      Some(ActionOutput {
        error: Some("error"),
        ..
      })
    ));
  }

  #[test]
  fn action_reject_if() {
    let action: Action<_> = simple(|_| 1).reject_if(|ctx| ctx.output.rest().len() > 0);

    assert!(matches!(
      action.exec(&mut ActionInput::new("A", 0, &mut ())),
      Some(ActionOutput { error: None, .. })
    ));
    assert!(matches!(
      action.exec(&mut ActionInput::new("AA", 0, &mut ())),
      None
    ));
  }

  #[test]
  fn action_reject() {
    let rejected_action: Action<_> = simple(|_| 1).reject();

    assert!(matches!(
      rejected_action.exec(&mut ActionInput::new("A", 0, &mut ())),
      None
    ));
  }

  #[test]
  fn action_callback() {
    // ensure callback can update the state
    let mut state = MyState { value: 0 };
    let action: Action<_, MyState, ()> = simple(|input: &ActionInput<MyState>| input.rest().len())
      .callback(|ctx| ctx.input.state.value += 1);

    // ensure may_mutate_state is set to true
    assert!(action.may_mutate_state);

    assert!(matches!(
      action.exec(&mut ActionInput::new("A", 0, &mut state)),
      Some(ActionOutput { .. })
    ));
    assert_eq!(state.value, 1);
  }
}
