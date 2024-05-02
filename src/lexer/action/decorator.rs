mod combine;
mod context;
mod data;
mod head;
mod kind;

pub use context::*;

use super::{input::ActionInput, output::ActionOutput, Action};

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  /// Check the [`ActionInput`] before the action is executed.
  /// Reject the action if the `condition` returns `true`.
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
  pub fn prevent(
    mut self,
    condition: impl Fn(
        // action state is immutable
        &ActionInput<ActionState>,
      ) -> bool
      + 'static,
  ) -> Self
  where
    ActionState: 'static,
    ErrorType: 'static,
  {
    let exec = self.exec;
    self.exec = Box::new(move |input| if condition(input) { None } else { exec(input) });
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
  pub fn prepare(
    mut self,
    modifier: impl Fn(
        // action state is mutable
        &mut ActionInput<ActionState>,
      ) + 'static,
  ) -> Self
  where
    ActionState: 'static,
    ErrorType: 'static,
  {
    let exec = self.exec;
    self.exec = Box::new(move |input| {
      modifier(input);
      exec(input)
    });
    self.may_mutate_state = true;
    self
  }

  /// Set [`Self::muted`] to `true`.
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
  pub fn mute(mut self) -> Self {
    self.muted = true;
    self
  }

  /// Set [`Self::muted`] to `false`.
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
  pub fn unmute(mut self) -> Self {
    self.muted = false;
    self
  }

  /// Set [`ActionOutput::error`] if the action is accepted.
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
  pub fn check<NewError>(
    self,
    condition: impl Fn(
        AcceptedActionOutputContext<
          &ActionInput<ActionState>,
          // user could consume the old error, but not able to consume the kind
          ActionOutput<&Kind, Option<ErrorType>>,
        >,
      ) -> Option<NewError>
      + 'static,
  ) -> Action<Kind, ActionState, NewError>
  where
    ActionState: 'static,
    ErrorType: 'static,
  {
    let exec = self.exec;
    Action {
      exec: Box::new(move |input| {
        exec(input).map(|output| ActionOutput {
          error: condition(AcceptedActionOutputContext {
            input,
            output: ActionOutput {
              kind: &output.kind,  // don't consume the kind
              error: output.error, // but the error is consumable
              digested: output.digested,
            },
          }),
          kind: output.kind,
          digested: output.digested,
        })
      }),
      may_mutate_state: self.may_mutate_state,
      muted: self.muted,
      head_matcher: self.head_matcher,
      kind_id: self.kind_id,
      literal: self.literal,
    }
  }

  /// Set [`ActionOutput::error`] if the action is accepted.
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
    ActionState: 'static,
    ErrorType: 'static,
    NewError: Clone + 'static,
  {
    let exec = self.exec;
    Action {
      exec: Box::new(move |input| {
        exec(input).map(|output| ActionOutput {
          error: Some(error.clone()),
          kind: output.kind,
          digested: output.digested,
        })
      }),
      may_mutate_state: self.may_mutate_state,
      muted: self.muted,
      head_matcher: self.head_matcher,
      kind_id: self.kind_id,
      literal: self.literal,
    }
  }

  /// Reject the action if the condition is met.
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
  ///     .reject_if(|ctx| ctx.rest().len() > 0)
  /// );
  /// ```
  pub fn reject_if(
    mut self,
    condition: impl Fn(
        // user should NOT mutate the output directly
        AcceptedActionOutputContext<
          &ActionInput<ActionState>,
          &ActionOutput<Kind, Option<ErrorType>>,
        >,
      ) -> bool
      + 'static,
  ) -> Self
  where
    ActionState: 'static,
    ErrorType: 'static,
  {
    let exec = self.exec;
    self.exec = Box::new(move |input| {
      exec(input).and_then(|output| {
        if condition(AcceptedActionOutputContext {
          input,
          output: &output,
        }) {
          None
        } else {
          output.into()
        }
      })
    });
    self
  }

  /// Reject the action after execution.
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
  pub fn reject(mut self) -> Self
  where
    ActionState: 'static,
    ErrorType: 'static,
  {
    let exec = self.exec;
    self.exec = Box::new(move |input| {
      exec(input);
      None
    });
    self
  }
  // `reject_if(move |_| false)` is meaningless
  // so there is no method like `un_reject`

  /// Call the `cb` if the action is accepted.
  /// You can modify [`ActionInput::state`] in the `cb`.
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
  ///   |a| a.callback(|ctx| ctx.input.state.value += 1)
  /// );
  /// ```
  pub fn callback(
    mut self,
    cb: impl Fn(
        AcceptedActionOutputContext<
          // user can mutate the input.state
          &mut ActionInput<ActionState>,
          // user should NOT mutate the output directly
          &ActionOutput<Kind, Option<ErrorType>>,
        >,
      ) + 'static,
  ) -> Self
  where
    ActionState: 'static,
    ErrorType: 'static,
  {
    let exec = self.exec;
    self.exec = Box::new(move |input| {
      exec(input).map(|output| {
        cb(AcceptedActionOutputContext {
          output: &output,
          input,
        });
        output
      })
    });
    self.may_mutate_state = true;
    self
  }

  /// Apply a decorator to this action.
  /// This is useful when you want to apply multiple decorators to multi actions.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, exact}};
  /// # let action: Action<_> =
  /// exact("a").apply(|a| a.mute().reject());
  /// ```
  // TODO: use blanket impl for all struct
  pub fn apply<T>(self, f: impl FnOnce(Self) -> T) -> T {
    f(self)
  }
}

#[cfg(test)]
mod tests {
  use crate::lexer::action::{
    exact, input::ActionInput, output::ActionOutput, AcceptedActionOutputContext, Action,
  };
  use whitehole_macros::_token_kind;

  #[_token_kind]
  #[derive(Clone)]
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
    let action: Action<_, _> = exact("a")
      // modify the state before the action is executed
      .prepare(|input: &mut ActionInput<MyState>| input.state.value += 1)
      // prevent the action if the rest is empty
      .prevent(|input| input.rest().len() == 0);

    // the first exec, state will be changed, digest all chars
    let output = action.exec(&mut ActionInput::new("a", 0, &mut state).unwrap());
    assert!(matches!(output, Some(ActionOutput { digested: 1, .. })));
    assert_eq!(state.value, 1);

    // the second exec, the action is prevented, so the state is not updated
    let output = action.exec(&mut ActionInput::new("a", 1, &mut state).unwrap());
    assert!(matches!(output, None));
    assert_eq!(state.value, 1); // the state is not updated
  }

  #[test]
  fn action_prepare() {
    let mut state = MyState { value: 0 };
    let action: Action<_, _> = exact("a")
      // modify the state before the action is executed
      .prepare(|input: &mut ActionInput<MyState>| input.state.value += 1);

    // ensure may_mutate_state is set to true
    assert!(action.may_mutate_state);

    // the action is rejected, but the state is still updated
    let output = action.exec(&mut ActionInput::new("b", 0, &mut state).unwrap());
    assert!(matches!(output, None));
    assert_eq!(state.value, 1);
  }

  #[test]
  fn action_mute_unmute() {
    let muted_action: Action<_> = exact("a").mute();
    let not_muted_action: Action<_> = exact("a").mute().unmute();

    assert!(muted_action.muted);
    assert!(!not_muted_action.muted);
  }

  #[test]
  fn action_check() {
    let action = exact("a").check(
      |ctx: AcceptedActionOutputContext<_, ActionOutput<_, Option<&str>>>| {
        if ctx.rest().len() > 0 {
          Some("error")
        } else {
          None
        }
      },
    );

    assert!(matches!(
      action.exec(&mut ActionInput::new("a", 0, &mut ()).unwrap()),
      Some(ActionOutput { error: None, .. })
    ));
    assert!(matches!(
      action.exec(&mut ActionInput::new("aa", 0, &mut ()).unwrap()),
      Some(ActionOutput {
        error: Some("error"),
        ..
      })
    ));
  }

  #[test]
  fn action_error() {
    let action = exact::<_, &str>("a").error("error");

    assert!(matches!(
      action.exec(&mut ActionInput::new("a", 0, &mut ()).unwrap()),
      Some(ActionOutput {
        error: Some("error"),
        ..
      })
    ));
  }

  #[test]
  fn action_reject_if() {
    let action: Action<_> = exact("a").reject_if(|ctx| ctx.rest().len() > 0);

    assert!(matches!(
      action.exec(&mut ActionInput::new("a", 0, &mut ()).unwrap()),
      Some(ActionOutput { error: None, .. })
    ));
    assert!(matches!(
      action.exec(&mut ActionInput::new("aa", 0, &mut ()).unwrap()),
      None
    ));
  }

  #[test]
  fn action_reject() {
    let rejected_action: Action<_> = exact("a").reject();

    assert!(matches!(
      rejected_action.exec(&mut ActionInput::new("a", 0, &mut ()).unwrap()),
      None
    ));
  }

  #[test]
  fn action_callback() {
    // ensure callback can update the state
    let mut state = MyState { value: 0 };
    let action: Action<_, MyState, ()> = exact("a").callback(
      |ctx: AcceptedActionOutputContext<&mut ActionInput<MyState>, _>| ctx.input.state.value += 1,
    );

    // ensure may_mutate_state is set to true
    assert!(action.may_mutate_state);

    assert!(matches!(
      action.exec(&mut ActionInput::new("a", 0, &mut state).unwrap()),
      Some(ActionOutput { .. })
    ));
    assert_eq!(state.value, 1);
  }
}
