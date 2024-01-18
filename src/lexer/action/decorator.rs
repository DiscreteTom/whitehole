use super::{
  input::ActionInput,
  output::{ActionOutput, EnhancedActionOutput},
  Action,
};

/// `input.state` is not mutable. `output` is consumed.
pub struct AcceptedActionDecoratorContext<'input, 'buffer, 'state, Kind, ActionState, ErrorType> {
  pub input: &'input ActionInput<'buffer, 'state, ActionState>,
  pub output: EnhancedActionOutput<'buffer, Kind, ErrorType>,
}

/// `input.state` is mutable. `output` is not mutable and not consumed.
pub struct ActionCallbackContext<'input, 'buffer, 'state, 'output, Kind, ActionState, ErrorType> {
  pub input: &'input mut ActionInput<'buffer, 'state, ActionState>,
  pub output: &'output EnhancedActionOutput<'buffer, Kind, ErrorType>,
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static> Action<Kind, ActionState, ErrorType> {
  /// Check the `ActionInput` before the action is executed.
  /// Reject the action if the `condition` returns `true`.
  /// Return a new action.
  pub fn prevent<F>(self, condition: F) -> Self
  where
    F: Fn(&ActionInput<ActionState>) -> bool + 'static,
  {
    let exec = self.exec;
    Action {
      exec: Box::new(
        move |input: &mut ActionInput<ActionState>| {
          if condition(input) {
            None
          } else {
            exec(input)
          }
        },
      ),
      maybe_muted: self.maybe_muted,
      possible_kinds: self.possible_kinds,
    }
  }

  /// Apply a decorator to this action.
  /// Usually used to modify the `ActionOutput`.
  /// Return a new action.
  pub fn apply<F, NewErrorType>(self, decorator: F) -> Action<Kind, ActionState, NewErrorType>
  where
    F: Fn(
        AcceptedActionDecoratorContext<Kind, ActionState, ErrorType>,
      ) -> Option<ActionOutput<Kind, NewErrorType>>
      + 'static,
  {
    let exec = self.exec;
    Action {
      exec: Box::new(
        move |input: &mut ActionInput<ActionState>| match exec(input) {
          Some(output) => decorator(AcceptedActionDecoratorContext {
            output: EnhancedActionOutput::new(input, output),
            input,
          }),
          None => None,
        },
      ),
      maybe_muted: self.maybe_muted,
      possible_kinds: self.possible_kinds,
    }
  }

  /// Set `ActionOutput.muted` if the action is accepted.
  /// Return a new action.
  pub fn mute_if<F>(self, condition: F) -> Self
  where
    F: Fn(&AcceptedActionDecoratorContext<Kind, ActionState, ErrorType>) -> bool + 'static,
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

  /// Set `ActionOutput.muted` if the action is accepted.
  /// Return a new action.
  pub fn mute(self, muted: bool) -> Self {
    let mut res = self.apply(move |mut ctx| {
      ctx.output.muted = muted;
      ctx.output.into()
    });
    res.maybe_muted = muted; // we know this
    res
  }

  /// Set `ActionOutput.error` if the action is accepted.
  /// Return a new action.
  pub fn check<F, NewError>(self, condition: F) -> Action<Kind, ActionState, NewError>
  where
    F: Fn(&AcceptedActionDecoratorContext<Kind, ActionState, ErrorType>) -> Option<NewError>
      + 'static,
  {
    self.apply(move |ctx| {
      Some(ActionOutput {
        error: condition(&ctx),
        kind: ctx.output.kind,
        digested: ctx.output.digested,
        muted: ctx.output.muted,
      })
    })
  }

  /// Set `ActionOutput.error` if the action is accepted.
  /// Return a new action.
  pub fn error<NewError>(self, error: NewError) -> Action<Kind, ActionState, NewError>
  where
    NewError: Clone + 'static,
  {
    self.check(move |_| Some(error.clone()))
  }

  /// Reject the action if the condition is met.
  /// Return a new action.
  pub fn reject_if<F>(self, condition: F) -> Self
  where
    F: Fn(&AcceptedActionDecoratorContext<Kind, ActionState, ErrorType>) -> bool + 'static,
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
  pub fn reject(self, rejected: bool) -> Self {
    self.reject_if(move |_| rejected)
  }

  /// Call the `callback` if the action is accepted and `peek` is `false`.
  /// You can modify the action state in the `callback`.
  /// Return a new action.
  pub fn then<F>(self, callback: F) -> Self
  where
    F: Fn(ActionCallbackContext<Kind, ActionState, ErrorType>) + 'static,
  {
    let exec = self.exec;
    Action {
      exec: Box::new(move |input| match exec(input) {
        Some(output) => {
          let output = EnhancedActionOutput::new(&input, output);
          if !input.peek() {
            callback(ActionCallbackContext {
              output: &output,
              input,
            });
          }
          output.into()
        }
        None => None,
      }),
      maybe_muted: self.maybe_muted,
      possible_kinds: self.possible_kinds,
    }
  }

  /// Execute another action if current action can't be accepted.
  /// Return a new action.
  pub fn or(self, another: Self) -> Self {
    let exec = self.exec;
    let another_exec = another.exec;
    Action {
      exec: Box::new(move |input| match exec(input) {
        Some(output) => Some(output),
        None => another_exec(input),
      }),
      maybe_muted: self.maybe_muted || another.maybe_muted,
      possible_kinds: self.possible_kinds, // these two actions should have the same possible kinds
    }
  }
}
