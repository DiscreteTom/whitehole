use super::{
  input::ActionInput,
  output::{ActionOutput, EnhancedActionOutput},
  Action, ActionInputRestHeadMatcher,
};
use crate::lexer::token::TokenKind;
use std::collections::HashSet;

/// `input.state` is mutable. `output` is consumed.
pub struct AcceptedActionDecoratorContext<'input, 'buffer, 'state, Kind, ActionState, ErrorType> {
  pub input: &'input mut ActionInput<'buffer, 'state, ActionState>,
  pub output: EnhancedActionOutput<'buffer, Kind, ErrorType>,
}

/// `input.state` is mutable. `output` is not mutable and not consumed.
pub struct ActionCallbackContext<'input, 'buffer, 'state, 'output, Kind, ActionState, ErrorType> {
  pub input: &'input mut ActionInput<'buffer, 'state, ActionState>,
  pub output: &'output EnhancedActionOutput<'buffer, Kind, ErrorType>,
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static> Action<Kind, ActionState, ErrorType> {
  /// Check the [`ActionInput`] before the action is executed.
  /// Reject the action if the `condition` returns `true`.
  /// Return a new action.
  pub fn prevent<F>(mut self, condition: F) -> Self
  where
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
  /// Return a new action.
  pub fn apply<NewErrorType, F>(self, decorator: F) -> Action<Kind, ActionState, NewErrorType>
  where
    F: Fn(
        AcceptedActionDecoratorContext<Kind, ActionState, ErrorType>,
      ) -> Option<ActionOutput<Kind, NewErrorType>>
      + 'static,
  {
    let exec = self.exec;
    Action {
      exec: Box::new(move |input: &mut ActionInput<ActionState>| {
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
  pub fn mute_if<F>(self, condition: F) -> Self
  where
    F: Fn(&AcceptedActionDecoratorContext<Kind, ActionState, ErrorType>) -> bool + 'static,
  {
    let mut res = self.apply(move |mut ctx| {
      ctx.output.raw.muted = condition(&ctx);
      ctx.output.into()
    });
    // we can't know whether the output will be muted
    // so we set maybe_muted to true
    res.maybe_muted = true;
    res
  }

  /// Set [`ActionOutput::muted`] if the action is accepted.
  /// Return a new action.
  pub fn mute(self, muted: bool) -> Self {
    // reminder: DON'T use `self.mute_if(move |_| muted)`
    // because we can set `maybe_muted` to `muted` directly
    let mut res = self.apply(move |mut ctx| {
      ctx.output.raw.muted = muted;
      ctx.output.into()
    });
    res.maybe_muted = muted; // we know this
    res
  }

  /// Set [`ActionOutput::error`] if the action is accepted.
  /// Return a new action.
  pub fn check<NewError, F>(self, condition: F) -> Action<Kind, ActionState, NewError>
  where
    F: Fn(&AcceptedActionDecoratorContext<Kind, ActionState, ErrorType>) -> Option<NewError>
      + 'static,
  {
    self.apply(move |ctx| {
      Some(ActionOutput {
        error: condition(&ctx),
        kind: ctx.output.raw.kind,
        digested: ctx.output.raw.digested,
        muted: ctx.output.raw.muted,
      })
    })
  }

  /// Set [`ActionOutput::error`] if the action is accepted.
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

  /// Call the `callback` if the action is accepted.
  /// Return a new action.
  pub fn then<F>(mut self, callback: F) -> Self
  where
    F: Fn(ActionCallbackContext<Kind, ActionState, ErrorType>) + 'static,
  {
    let exec = self.exec;
    self.exec = Box::new(move |input| {
      exec(input).and_then(|output| {
        let output = EnhancedActionOutput::new(&input, output);
        callback(ActionCallbackContext {
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
  pub fn or(mut self, another: Self) -> Self {
    let exec = self.exec;
    let another_exec = another.exec;
    self.exec = Box::new(move |input| exec(input).or_else(|| another_exec(input)));
    self.maybe_muted = self.maybe_muted || another.maybe_muted;
    self.possible_kinds.extend(another.possible_kinds); // merge possible kinds
    self
  }

  /// Set the kind and the data binding for this action.
  /// Use this if your action can only yield one kind.
  pub fn bind<NewKind>(self, kind: impl Into<NewKind>) -> Action<NewKind, ActionState, ErrorType>
  where
    NewKind: TokenKind<NewKind> + Clone + 'static,
  {
    let kind = kind.into();
    self.kind_ids([kind.id()]).select(move |_| kind.clone())
  }

  /// Set [`Action::head_matcher`] to [`OneOf`](ActionInputRestHeadMatcher::OneOf).
  pub fn head_in(mut self, char_set: impl Into<HashSet<char>>) -> Self {
    self.head_matcher = Some(ActionInputRestHeadMatcher::OneOf(char_set.into()));
    self
  }

  /// Set [`Action::head_matcher`] to [`Not`](ActionInputRestHeadMatcher::Not).
  pub fn head_not(mut self, char_set: impl Into<HashSet<char>>) -> Self {
    self.head_matcher = Some(ActionInputRestHeadMatcher::Not(char_set.into()));
    self
  }

  // there is no `Action.map` or `Action.data` like in retsac since rust doesn't support value-level type or type union,
  // so we have to provide `possible_kinds` manually if we implement `Action.map` or `Action.data`,
  // which is the same as calling `action.kinds().select()`.
}
