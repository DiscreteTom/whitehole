mod context;
mod data;
mod head;
mod kind;
mod literal;

pub use context::*;

use super::{input::ActionInput, output::ActionOutput, Action, ActionExec, RawActionExec};
use crate::lexer::token::TokenKindId;

/// Apply a statement and return `self`.
macro_rules! echo_with {
  ($self:expr, $s:stmt) => {{
    $s
    $self
  }};
}
pub(super) use echo_with;

// simple decorators that doesn't require generic bounds
impl<Kind, State, Heap> Action<Kind, State, Heap> {
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
    echo_with!(self, self.muted = true)
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
    echo_with!(self, self.muted = false)
  }
}

// these decorators will use `Box` to construct new action exec
// so `Kind/State/Heap` must be `'static`
impl<Kind: 'static, State: 'static, Heap: 'static> Action<Kind, State, Heap> {
  /// Apply a function to [`Action::exec`] and return the modified `self`.
  #[inline]
  fn map_exec(
    mut self,
    f: impl Fn(
        &RawActionExec<Kind, State, Heap>,
        &mut ActionInput<&mut State, &mut Heap>,
      ) -> Option<ActionOutput<Kind>>
      + 'static,
  ) -> Self {
    echo_with!(self, {
      let exec = self.exec.raw;
      self.exec = ActionExec::new(move |input| f(&exec, input));
    })
  }

  /// Apply a function to [`Action::exec`] and return a new [`Action`]
  /// with a different `Kind`.
  #[inline]
  fn map_exec_new<NewKind>(
    self,
    kind: TokenKindId<NewKind>,
    f: impl Fn(
        &RawActionExec<Kind, State, Heap>,
        &mut ActionInput<&mut State, &mut Heap>,
      ) -> Option<ActionOutput<NewKind>>
      + 'static,
  ) -> Action<NewKind, State, Heap> {
    let exec = self.exec.raw;
    Action {
      exec: ActionExec::new(move |input| f(&exec, input)),
      muted: self.muted,
      head: self.head,
      kind,
      literal: self.literal,
    }
  }

  /// Check the [`ActionInput`] before the action is executed.
  /// Reject the action if the `condition` returns `true`.
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
  #[inline]
  pub fn prevent(
    self,
    condition: impl Fn(&mut ActionInput<&mut State, &mut Heap>) -> bool + 'static,
  ) -> Self {
    self.map_exec(
      move |exec, input| {
        if condition(input) {
          None
        } else {
          exec(input)
        }
      },
    )
  }

  /// Modify `State` and `Heap` before the action is executed.
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
  #[inline]
  pub fn prepare(
    self,
    modifier: impl Fn(&mut ActionInput<&mut State, &mut Heap>) + 'static,
  ) -> Self {
    self.map_exec(move |exec, input| {
      modifier(input);
      exec(input)
    })
  }

  /// Reject the action if the `condition` returns `true`.
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
  #[inline]
  pub fn reject_if(
    self,
    condition: impl Fn(
        AcceptedActionOutputContext<&mut ActionInput<&mut State, &mut Heap>, &ActionOutput<Kind>>,
      ) -> bool
      + 'static,
  ) -> Self {
    self.map_exec(move |exec, input| {
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
    })
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
  #[inline]
  pub fn reject(self) -> Self {
    // to optimize the runtime performance,
    // don't just use `reject_if(|_| true)`
    // to prevent constructing the context

    self.map_exec(move |exec, input| {
      exec(input);
      None
    })
  }
  // `reject_if(|_| false)` is meaningless
  // so there is no method like `un_reject`

  /// Call the `cb` if the action is accepted.
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
  ///   |a| a.then(|ctx| ctx.input.state.value += 1)
  /// );
  /// # }
  /// ```
  #[inline]
  pub fn then(
    self,
    cb: impl Fn(AcceptedActionOutputContext<&mut ActionInput<&mut State, &mut Heap>, &ActionOutput<Kind>>)
      + 'static,
  ) -> Self {
    self.map_exec(move |exec, input| {
      exec(input).map(|output| {
        cb(AcceptedActionOutputContext {
          input,
          output: &output,
        });
        output
      })
    })
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
      .prepare(|input: &mut ActionInput<&mut MyState, &mut ()>| input.state.value += 1)
      // prevent the action if the rest is empty
      .prevent(|input| input.rest().len() == 1);

    // the first exec, state will be changed, digest all chars
    let output = (action.exec.raw)(&mut ActionInput::new("aa", 0, &mut state, &mut ()).unwrap());
    assert!(matches!(output, Some(ActionOutput { digested: 1, .. })));
    assert_eq!(state.value, 1);

    // the second exec, the action is prevented, so the state is not updated
    let output = (action.exec.raw)(&mut ActionInput::new("aa", 1, &mut state, &mut ()).unwrap());
    assert!(matches!(output, None));
    assert_eq!(state.value, 1); // the state is not updated

    // prevent for immutable action
    let action: Action<_> = exact("a").prevent(|_| true);
    assert!((action.exec.raw)(&mut ActionInput::new("a", 0, &mut (), &mut ()).unwrap()).is_none());
  }

  #[test]
  fn action_prepare() {
    let mut state = MyState { value: 0 };
    let action: Action<_, _> = exact("a")
      // modify the state before the action is executed
      .prepare(|input: &mut ActionInput<&mut MyState, &mut ()>| input.state.value += 1);

    // the action is rejected, but the state is still updated
    let output = (action.exec.raw)(&mut ActionInput::new("b", 0, &mut state, &mut ()).unwrap());
    assert!(matches!(output, None));
    assert_eq!(state.value, 1);

    // prepare for mutable action
    let action = action.prepare(|input| input.state.value += 1);
    state.value = 0;
    let output = (action.exec.raw)(&mut ActionInput::new("b", 0, &mut state, &mut ()).unwrap());
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
  fn action_reject_if() {
    let action: Action<_> = exact("a").reject_if(|ctx| ctx.rest().len() > 0);

    assert!((action.exec.raw)(&mut ActionInput::new("a", 0, &mut (), &mut ()).unwrap()).is_some());
    assert!(matches!(
      (action.exec.raw)(&mut ActionInput::new("aa", 0, &mut (), &mut ()).unwrap()),
      None
    ));

    // reject for mutable action
    let action: Action<_, i32> = exact("a")
      .prepare(|input| *input.state += 1)
      .reject_if(|ctx| ctx.rest().len() > 0);
    let mut state = 0;
    assert!(matches!(
      (action.exec.raw)(&mut ActionInput::new("a ", 0, &mut state, &mut ()).unwrap()),
      None
    ));
    assert_eq!(state, 1);
  }

  #[test]
  fn action_reject() {
    let rejected_action: Action<_> = exact("a").reject();

    assert!(matches!(
      (rejected_action.exec.raw)(&mut ActionInput::new("a", 0, &mut (), &mut ()).unwrap()),
      None
    ));

    // reject for mutable action
    let action: Action<_, i32> = exact("a").prepare(|input| *input.state += 1).reject();
    let mut state = 0;
    assert!(matches!(
      (action.exec.raw)(&mut ActionInput::new("a ", 0, &mut state, &mut ()).unwrap()),
      None
    ));
    assert_eq!(state, 1);
  }

  #[test]
  fn action_callback() {
    // ensure callback can update the state
    let mut state = MyState { value: 0 };
    let action: Action<_, MyState> = exact("a").then(
      |ctx: AcceptedActionOutputContext<&mut ActionInput<&mut MyState, &mut ()>, _>| {
        ctx.input.state.value += 1
      },
    );

    assert!(matches!(
      (action.exec.raw)(&mut ActionInput::new("a", 0, &mut state, &mut ()).unwrap()),
      Some(ActionOutput { .. })
    ));
    assert_eq!(state.value, 1);

    // callback for mutable action
    let action = action.then(|ctx| ctx.input.state.value += 1);
    state.value = 0;
    assert!(matches!(
      (action.exec.raw)(&mut ActionInput::new("a", 0, &mut state, &mut ()).unwrap()),
      Some(ActionOutput { .. })
    ));
    assert_eq!(state.value, 2);
  }
}
