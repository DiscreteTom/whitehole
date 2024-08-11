mod context;
mod data;
mod head;
mod kind;
mod literal;

pub use context::*;

use super::{input::ActionInput, mut_input_to_ref, output::ActionOutput, Action, ActionExec};
use crate::lexer::{
  action::{map_exec, map_exec_adapt_input},
  token::TokenKindIdBinding,
};

/// Convert the content of [`ActionExec`] to [`ActionExec::Mutable`] using the given macro.
/// The `macro_impl`'s second argument is a boolean indicating
/// whether to convert the mutable action to immutable.
macro_rules! map_exec_to_mut {
  ($exec: expr, $macro_impl: ident) => {
    match $exec {
      // convert immutable to mutable
      ActionExec::Immutable(exec) => ActionExec::Mutable($macro_impl!(exec, true)),
      ActionExec::Mutable(exec) => ActionExec::Mutable($macro_impl!(exec, false)),
    }
  };
}

// simple decorators that doesn't require generic bounds
impl<Kind, State, ErrorType> Action<Kind, State, ErrorType> {
  /// Set [`Self::muted`] to `true`.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::regex, LexerBuilder, token::token_kind};
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
  pub fn mute(mut self) -> Self {
    self.muted = true;
    self
  }

  /// Set [`Self::muted`] to `false`.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::regex, LexerBuilder, token::token_kind};
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
  pub fn unmute(mut self) -> Self {
    self.muted = false;
    self
  }
}

// these decorators will use `Box` to construct new action exec
// so `Kind/State/ErrorType` must be `'static`
impl<Kind: 'static, State: 'static, ErrorType: 'static> Action<Kind, State, ErrorType> {
  /// Check the [`ActionInput`] before the action is executed.
  /// Reject the action if the `condition` returns `true`.
  ///
  /// [`ActionInput::state`] is immutable in the `condition`.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::regex, LexerBuilder, token::token_kind};
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
  pub fn prevent(mut self, condition: impl Fn(&ActionInput<&State>) -> bool + 'static) -> Self {
    macro_rules! impl_prevent {
      ($exec: ident, $mut_input_to_ref: ident) => {
        Box::new(move |input| {
          if condition(mut_input_to_ref!(input, $mut_input_to_ref)) {
            None
          } else {
            $exec(input)
          }
        })
      };
    }

    self.exec = map_exec_adapt_input!(self.exec, impl_prevent);
    self
  }

  /// Modify `State` before the action is executed.
  /// [`ActionInput::state`] is mutable in the `modifier`.
  /// This will convert [`Self::exec`] to [`ActionExec::Mutable`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::regex, LexerBuilder, token::token_kind};
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
  pub fn prepare(mut self, modifier: impl Fn(&mut ActionInput<&mut State>) + 'static) -> Self {
    macro_rules! impl_prepare {
      ($exec: ident, $mut_input_to_ref: ident) => {
        Box::new(move |input| {
          modifier(input);
          $exec(mut_input_to_ref!(input, $mut_input_to_ref))
        })
      };
    }

    self.exec = map_exec_to_mut!(self.exec, impl_prepare);
    self
  }

  /// Set [`ActionOutput::error`] by the `factory` if the action is accepted.
  /// You can consume the old [`ActionOutput::error`] in the `factory`
  /// but not the [`ActionOutput::binding`].
  ///
  /// [`ActionInput::state`] is immutable in the `factory`.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::regex, LexerBuilder, token::token_kind};
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
    factory: impl Fn(
        AcceptedActionOutputContext<
          &ActionInput<&State>,
          ActionOutput<&TokenKindIdBinding<Kind>, Option<ErrorType>>,
        >,
      ) -> Option<NewError>
      + 'static,
  ) -> Action<Kind, State, NewError> {
    macro_rules! impl_check {
      ($exec: ident, $mut_input_to_ref: ident) => {
        Box::new(move |input| {
          $exec(input).map(|output| ActionOutput {
            error: factory(AcceptedActionOutputContext {
              input: mut_input_to_ref!(input, $mut_input_to_ref),
              output: ActionOutput {
                binding: &output.binding, // don't consume the binding
                error: output.error,      // but the error is consumable
                digested: output.digested,
              },
            }),
            binding: output.binding,
            digested: output.digested,
          })
        })
      };
    }

    Action {
      exec: map_exec_adapt_input!(self.exec, impl_check),
      muted: self.muted,
      head: self.head,
      kind: self.kind,
      literal: self.literal,
    }
  }

  /// Set [`ActionOutput::error`] to a constant value if the action is accepted.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::regex, LexerBuilder, token::token_kind};
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
  pub fn error<NewError>(self, error: NewError) -> Action<Kind, State, NewError>
  where
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
            binding: output.binding,
            digested: output.digested,
          })
        })
      };
    }

    Action {
      exec: map_exec!(self.exec, impl_error),
      muted: self.muted,
      head: self.head,
      kind: self.kind,
      literal: self.literal,
    }
  }

  /// Reject the action if the `condition` is met.
  ///
  /// [`ActionInput::state`] is immutable in the `condition`.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::regex, LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// builder.define_with(
  ///   A,
  ///   regex(r"^\s+"),
  ///   |a| a.reject_if(|ctx| ctx.rest().len() > 0)
  /// );
  /// # }
  /// ```
  pub fn reject_if(
    mut self,
    condition: impl Fn(
        AcceptedActionOutputContext<
          &ActionInput<&State>,
          &ActionOutput<TokenKindIdBinding<Kind>, Option<ErrorType>>,
        >,
      ) -> bool
      + 'static,
  ) -> Self {
    macro_rules! impl_reject_if {
      ($exec: ident, $mut_input_to_ref: ident) => {
        Box::new(move |input| {
          $exec(input).and_then(|output| {
            if condition(AcceptedActionOutputContext {
              input: mut_input_to_ref!(input, $mut_input_to_ref),
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

    self.exec = map_exec_adapt_input!(self.exec, impl_reject_if);
    self
  }

  /// Reject the action after execution.
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::regex, LexerBuilder, token::token_kind};
  /// # #[token_kind]
  /// # #[derive(Clone)]
  /// # enum MyKind { A }
  /// # fn main() {
  /// # let mut builder = LexerBuilder::new();
  /// builder.define_with(
  ///   A,
  ///   regex(r"^\s+"),
  ///   |a| a.reject()
  /// );
  /// # }
  /// ```
  pub fn reject(mut self) -> Self {
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

    self.exec = map_exec!(self.exec, impl_reject);
    self
  }
  // `reject_if(|_| false)` is meaningless
  // so there is no method like `un_reject`

  /// Call the `cb` if the action is accepted.
  /// You can modify [`ActionInput::state`] in the `cb`.
  /// This will set [`Self::exec`] to [`ActionExec::Mutable`].
  /// # Examples
  /// ```
  /// # use whitehole::lexer::{action::regex, LexerBuilder, token::token_kind};
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
          &mut ActionInput<&mut State>,
          &ActionOutput<TokenKindIdBinding<Kind>, Option<ErrorType>>,
        >,
      ) + 'static,
  ) -> Self {
    macro_rules! impl_callback {
      ($exec: ident, $mut_input_to_ref: ident) => {
        Box::new(move |input| {
          $exec(mut_input_to_ref!(input, $mut_input_to_ref)).map(|output| {
            cb(AcceptedActionOutputContext {
              input,
              output: &output,
            });
            output
          })
        })
      };
    }

    self.exec = map_exec_to_mut!(self.exec, impl_callback);
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

    // prevent for immutable action
    let action: Action<_> = exact("a").prevent(|_| true);
    assert!(action.exec.as_immutable()(&ActionInput::new("a", 0, &()).unwrap()).is_none());
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

    // prepare for mutable action
    let action = action.prepare(|input| input.state.value += 1);
    state.value = 0;
    let output = action.exec.as_mutable()(&mut ActionInput::new("b", 0, &mut state).unwrap());
    assert!(matches!(output, None));
    assert_eq!(state.value, 2);
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

    // reject for mutable action
    let action: Action<_, i32> = exact("a")
      .prepare(|input| *input.state += 1)
      .reject_if(|ctx| ctx.rest().len() > 0);
    let mut state = 0;
    assert!(matches!(
      action.exec.as_mutable()(&mut ActionInput::new("a ", 0, &mut state).unwrap()),
      None
    ));
    assert_eq!(state, 1);
  }

  #[test]
  fn action_reject() {
    let rejected_action: Action<_> = exact("a").reject();

    assert!(matches!(
      rejected_action.exec.as_immutable()(&ActionInput::new("a", 0, &()).unwrap()),
      None
    ));

    // reject for mutable action
    let action: Action<_, i32> = exact("a").prepare(|input| *input.state += 1).reject();
    let mut state = 0;
    assert!(matches!(
      action.exec.as_mutable()(&mut ActionInput::new("a ", 0, &mut state).unwrap()),
      None
    ));
    assert_eq!(state, 1);
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

    // callback for mutable action
    let action = action.callback(|ctx| ctx.input.state.value += 1);
    state.value = 0;
    assert!(matches!(
      action.exec.as_mutable()(&mut ActionInput::new("a", 0, &mut state).unwrap()),
      Some(ActionOutput { .. })
    ));
    assert_eq!(state.value, 2);
  }
}
