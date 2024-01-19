use super::{
  decorator::AcceptedActionDecoratorContext,
  input::ActionInput,
  output::{ActionOutput, EnhancedActionOutput},
  Action,
};
use std::collections::HashSet;

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static> Action<Kind, ActionState, ErrorType> {
  /// Set kinds for this action. This is used if your action can yield multiple kinds.
  pub fn kinds<NewKind>(
    self,
    possible_kinds: HashSet<NewKind>,
  ) -> MultiKindAction<NewKind, Kind, ActionState, ErrorType> {
    MultiKindAction {
      possible_kinds,
      maybe_muted: self.maybe_muted,
      exec: self.exec,
    }
  }
}

pub struct MultiKindAction<NewKind, Kind, ActionState, ErrorType> {
  possible_kinds: HashSet<NewKind>,
  maybe_muted: bool,
  exec: Box<dyn Fn(&mut ActionInput<ActionState>) -> Option<ActionOutput<Kind, ErrorType>>>,
}

impl<NewKind, Kind: 'static, ActionState: 'static, ErrorType: 'static>
  MultiKindAction<NewKind, Kind, ActionState, ErrorType>
{
  pub fn new(
    possible_kinds: HashSet<NewKind>,
    maybe_muted: bool,
    exec: Box<dyn Fn(&mut ActionInput<ActionState>) -> Option<ActionOutput<Kind, ErrorType>>>,
  ) -> Self {
    MultiKindAction {
      possible_kinds,
      maybe_muted,
      exec,
    }
  }

  /// Define a selector to select a kind from action's kinds by action's input and output.
  pub fn select<F>(self, selector: F) -> Action<NewKind, ActionState, ErrorType>
  where
    F: Fn(AcceptedActionDecoratorContext<Kind, ActionState, ErrorType>) -> NewKind + 'static,
  {
    let exec = self.exec;
    Action {
      exec: Box::new(
        move |input: &mut ActionInput<ActionState>| match exec(input) {
          Some(output) => {
            let ctx = AcceptedActionDecoratorContext {
              output: EnhancedActionOutput::new(input, output),
              input,
            };
            Some(ActionOutput {
              kind: selector(ctx),
              digested: ctx.output.digested,
              muted: ctx.output.muted,
              error: ctx.output.error,
            })
          }
          None => None,
        },
      ),
      maybe_muted: self.maybe_muted,
      possible_kinds: self.possible_kinds,
    }
  }
}
