mod context;
mod data;
mod head;
mod kind;

pub use context::*;

use super::{action_input_to_ref, input::ActionInput, output::ActionOutput, Action, ActionExec};

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  /// Check the [`ActionInput`] before the action is executed.
  /// Reject the action if the `condition` returns `true`.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # #[derive(Clone, Default)]
  /// # struct MyState {
  /// #   pub reject: bool,
  /// # }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::stateful::<MyState>();
  /// builder.define_with(
  ///   A,
  ///   regex(r"^\s+"),
  ///   |a| a.prevent(|input| input.state.reject)
  /// );
  /// # }
  /// ```
  pub fn prevent(
    mut self,
    condition: impl Fn(
        // action state is immutable, so use immutable reference
        &ActionInput<&ActionState>,
      ) -> bool
      + 'static,
  ) -> Self
  where
    ActionState: 'static,
    ErrorType: 'static,
  {
    macro_rules! impl_prevent {
      ($exec: ident, $to_mutable: ident) => {
        Box::new(move |input| {
          if condition(action_input_to_ref!(input, $to_mutable)) {
            None
          } else {
            $exec(input)
          }
        })
      };
    }

    self.exec = match self.exec {
      ActionExec::Immutable(exec) => ActionExec::Immutable(impl_prevent!(exec, false)),
      ActionExec::Mutable(exec) => ActionExec::Mutable(impl_prevent!(exec, true)),
    };
    self
  }

  /// Modify `ActionState` before the action is executed.
  /// This will set [`Self::may_mutate_state`] to `true`.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # #[derive(Clone, Default)]
  /// # struct MyState {
  /// #   pub value: i32,
  /// # }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::stateful::<MyState>();
  /// builder.define_with(
  ///   A,
  ///   regex(r"^\s+"),
  ///   |a| a.prepare(|input| input.state.value += 1)
  /// );
  /// # }
  /// ```
  pub fn prepare(
    mut self,
    modifier: impl Fn(
        // action state is mutable
        &mut ActionInput<&mut ActionState>,
      ) + 'static,
  ) -> Self
  where
    ActionState: 'static,
    ErrorType: 'static,
  {
    macro_rules! impl_prepare {
      ($exec: ident, $to_mutable: ident) => {
        Box::new(move |input| {
          modifier(input);
          $exec(action_input_to_ref!(input, $to_mutable))
        })
      };
    }

    self.exec = match self.exec {
      ActionExec::Immutable(exec) => ActionExec::Mutable(impl_prepare!(exec, true)),
      ActionExec::Mutable(exec) => ActionExec::Mutable(impl_prepare!(exec, false)),
    };
    self
  }

  /// Set [`Self::muted`] to `true`.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// builder.define(
  ///   A,
  ///   regex(r"^\s+").mute()
  /// );
  /// # }
  /// ```
  #[inline]
  pub const fn mute(mut self) -> Self {
    self.muted = true;
    self
  }

  /// Set [`Self::muted`] to `false`.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// builder.define(
  ///   A,
  ///   regex(r"^\s+").unmute()
  /// );
  /// # }
  /// ```
  #[inline]
  pub const fn unmute(mut self) -> Self {
    self.muted = false;
    self
  }

  /// Set [`ActionOutput::error`] if the action is accepted.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::with_error();
  /// builder.define_with(
  ///   A,
  ///   regex(r"^\s+"),
  ///   |a| a.check(|ctx| {
  ///     if ctx.rest().len() > 0 {
  ///       Some("error")
  ///     } else {
  ///       None
  ///     }
  ///   })
  /// );
  /// # }
  /// ```
  pub fn check<NewError>(
    self,
    condition: impl Fn(
        AcceptedActionOutputContext<
          &ActionInput<&ActionState>,
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
    macro_rules! impl_check {
      ($exec: ident, $to_mutable: ident) => {
        Box::new(move |input| {
          $exec(input).map(|output| ActionOutput {
            error: condition(AcceptedActionOutputContext {
              input: action_input_to_ref!(input, $to_mutable),
              output: ActionOutput {
                kind: &output.kind,  // don't consume the kind
                error: output.error, // but the error is consumable
                digested: output.digested,
              },
            }),
            kind: output.kind,
            digested: output.digested,
          })
        })
      };
    }

    Action {
      exec: match self.exec {
        ActionExec::Immutable(exec) => ActionExec::Immutable(impl_check!(exec, false)),
        ActionExec::Mutable(exec) => ActionExec::Mutable(impl_check!(exec, true)),
      },
      muted: self.muted,
      head: self.head,
      kind: self.kind,
      literal: self.literal,
    }
  }

  /// Set [`ActionOutput::error`] if the action is accepted.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::with_error();
  /// builder.define_with(
  ///   A,
  ///   regex(r"^\s+"),
  ///   |a| a.error("error")
  /// );
  /// # }
  /// ```
  pub fn error<NewError>(self, error: NewError) -> Action<Kind, ActionState, NewError>
  where
    ActionState: 'static,
    ErrorType: 'static,
    NewError: Clone + 'static,
  {
    // to optimize the runtime performance,
    // don't just use `check(|_| Some(error.clone()))`
    // to prevent constructing the context

    macro_rules! impl_error {
      ($exec: ident) => {
        Box::new(move |input| {
          $exec(input).map(|output| ActionOutput {
            error: Some(error.clone()),
            kind: output.kind,
            digested: output.digested,
          })
        })
      };
    }

    Action {
      exec: match self.exec {
        ActionExec::Immutable(exec) => ActionExec::Immutable(impl_error!(exec)),
        ActionExec::Mutable(exec) => ActionExec::Mutable(impl_error!(exec)),
      },
      muted: self.muted,
      head: self.head,
      kind: self.kind,
      literal: self.literal,
    }
  }

  /// Reject the action if the condition is met.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// builder.define(
  ///   A,
  ///   regex(r"^\s+")
  ///     .reject_if(|ctx| ctx.rest().len() > 0)
  /// );
  /// # }
  /// ```
  pub fn reject_if(
    mut self,
    condition: impl Fn(
        // user should NOT mutate the output directly
        AcceptedActionOutputContext<
          &ActionInput<&ActionState>,
          &ActionOutput<Kind, Option<ErrorType>>,
        >,
      ) -> bool
      + 'static,
  ) -> Self
  where
    ActionState: 'static,
    ErrorType: 'static,
  {
    macro_rules! impl_reject_if {
      ($exec: ident, $to_mutable: ident) => {
        Box::new(move |input| {
          $exec(input).and_then(|output| {
            if condition(AcceptedActionOutputContext {
              input: action_input_to_ref!(input, $to_mutable),
              output: &output,
            }) {
              None
            } else {
              output.into()
            }
          })
        })
      };
    }

    self.exec = match self.exec {
      ActionExec::Immutable(exec) => ActionExec::Immutable(impl_reject_if!(exec, false)),
      ActionExec::Mutable(exec) => ActionExec::Mutable(impl_reject_if!(exec, true)),
    };
    self
  }

  /// Reject the action after execution.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// builder.define(
  ///   A,
  ///   regex(r"^\s+").reject()
  /// );
  /// # }
  /// ```
  pub fn reject(mut self) -> Self
  where
    ActionState: 'static,
    ErrorType: 'static,
  {
    // to optimize the runtime performance,
    // don't just use `reject_if(|_| true)`
    // to prevent constructing the context

    macro_rules! impl_reject {
      ($exec: ident) => {
        Box::new(move |input| {
          $exec(input);
          None
        })
      };
    }

    self.exec = match self.exec {
      ActionExec::Immutable(exec) => ActionExec::Immutable(impl_reject!(exec)),
      ActionExec::Mutable(exec) => ActionExec::Mutable(impl_reject!(exec)),
    };
    self
  }
  // `reject_if(|_| false)` is meaningless
  // so there is no method like `un_reject`

  /// Call the `cb` if the action is accepted.
  /// You can modify [`ActionInput::state`] in the `cb`.
  /// This will set [`Self::may_mutate_state`] to `true`.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::{Action, regex}, LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # #[derive(Clone, Default)]
  /// # struct MyState {
  /// #   pub value: i32,
  /// # }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::stateful::<MyState>();
  /// builder.define_with(
  ///   A,
  ///   regex(r"^\s+"),
  ///   |a| a.callback(|ctx| ctx.input.state.value += 1)
  /// );
  /// # }
  /// ```
  pub fn callback(
    mut self,
    cb: impl Fn(
        AcceptedActionOutputContext<
          // user can mutate the input.state
          &mut ActionInput<&mut ActionState>,
          // user should NOT mutate the output directly
          &ActionOutput<Kind, Option<ErrorType>>,
        >,
      ) + 'static,
  ) -> Self
  where
    ActionState: 'static,
    ErrorType: 'static,
  {
    macro_rules! impl_callback {
      ($exec: ident, $to_mutable: ident) => {
        Box::new(move |input| {
          $exec(action_input_to_ref!(input, $to_mutable)).map(|output| {
            cb(AcceptedActionOutputContext {
              output: &output,
              input,
            });
            output
          })
        })
      };
    }

    self.exec = match self.exec {
      ActionExec::Immutable(exec) => ActionExec::Mutable(impl_callback!(exec, true)),
      ActionExec::Mutable(exec) => ActionExec::Mutable(impl_callback!(exec, false)),
    };
    self
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
      .prepare(|input: &mut ActionInput<&mut MyState>| input.state.value += 1)
      // prevent the action if the rest is empty
      .prevent(|input| input.rest().len() == 1);

    // the first exec, state will be changed, digest all chars
    let output = action.exec.as_mutable()(&mut ActionInput::new("aa", 0, &mut state).unwrap());
    assert!(matches!(output, Some(ActionOutput { digested: 1, .. })));
    assert_eq!(state.value, 1);

    // the second exec, the action is prevented, so the state is not updated
    let output = action.exec.as_mutable()(&mut ActionInput::new("aa", 1, &mut state).unwrap());
    assert!(matches!(output, None));
    assert_eq!(state.value, 1); // the state is not updated
  }

  #[test]
  fn action_prepare() {
    let mut state = MyState { value: 0 };
    let action: Action<_, _> = exact("a")
      // modify the state before the action is executed
      .prepare(|input: &mut ActionInput<&mut MyState>| input.state.value += 1);

    // the action is rejected, but the state is still updated
    let output = action.exec.as_mutable()(&mut ActionInput::new("b", 0, &mut state).unwrap());
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
      action.exec.as_immutable()(&ActionInput::new("a", 0, &()).unwrap()),
      Some(ActionOutput { error: None, .. })
    ));
    assert!(matches!(
      action.exec.as_immutable()(&ActionInput::new("aa", 0, &()).unwrap()),
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
      action.exec.as_immutable()(&ActionInput::new("a", 0, &()).unwrap()),
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
      action.exec.as_immutable()(&ActionInput::new("a", 0, &()).unwrap()),
      Some(ActionOutput { error: None, .. })
    ));
    assert!(matches!(
      action.exec.as_immutable()(&ActionInput::new("aa", 0, &()).unwrap()),
      None
    ));
  }

  #[test]
  fn action_reject() {
    let rejected_action: Action<_> = exact("a").reject();

    assert!(matches!(
      rejected_action.exec.as_immutable()(&ActionInput::new("a", 0, &()).unwrap()),
      None
    ));
  }

  #[test]
  fn action_callback() {
    // ensure callback can update the state
    let mut state = MyState { value: 0 };
    let action: Action<_, MyState, ()> = exact("a").callback(
      |ctx: AcceptedActionOutputContext<&mut ActionInput<&mut MyState>, _>| {
        ctx.input.state.value += 1
      },
    );

    assert!(matches!(
      action.exec.as_mutable()(&mut ActionInput::new("a", 0, &mut state).unwrap()),
      Some(ActionOutput { .. })
    ));
    assert_eq!(state.value, 1);
  }
}
