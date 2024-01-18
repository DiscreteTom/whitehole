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
      action: self,
    }
  }
}

pub struct MultiKindAction<NewKind, Kind, ActionState, ErrorType> {
  possible_kinds: HashSet<NewKind>,
  action: Action<Kind, ActionState, ErrorType>,
}

impl<NewKinds, Kind: 'static, ActionState: 'static, ErrorType: 'static>
  MultiKindAction<NewKinds, Kind, ActionState, ErrorType>
{
  /// Define a selector to select a kind from action's kinds by action's input and output.
  pub fn select<F>(self, selector: F) -> Action<NewKinds, ActionState, ErrorType>
  where
    F: Fn(&AcceptedActionDecoratorContext<Kind, ActionState, ErrorType>) -> NewKinds + 'static,
  {
    let exec = self.action.exec;
    Action {
      exec: Box::new(
        move |input: &mut ActionInput<ActionState>| match exec(input) {
          Some(output) => {
            let ctx = AcceptedActionDecoratorContext {
              output: EnhancedActionOutput::new(input, output),
              input,
            };
            Some(ActionOutput {
              kind: selector(&ctx),
              digested: ctx.output.digested,
              muted: ctx.output.muted,
              error: ctx.output.error,
            })
          }
          None => None,
        },
      ),
      maybe_muted: self.action.maybe_muted,
      possible_kinds: self.possible_kinds,
    }
  }
}
