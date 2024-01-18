pub mod input;
pub mod output;
pub mod regex;
pub mod simple;

use self::{
  input::ActionInput,
  output::{ActionOutput, EnhancedActionOutput},
};
use std::collections::HashSet;

pub struct AcceptedActionDecoratorContext<'input, 'buffer, 'state, Kind, ActionState, ErrorType> {
  pub input: &'input mut ActionInput<'buffer, 'state, ActionState>,
  pub output: EnhancedActionOutput<'buffer, Kind, ErrorType>,
}

pub struct Action<Kind, ActionState, ErrorType> {
  /// This flag is to indicate whether this action's output might be muted.
  /// The lexer will based on this flag to accelerate the lexing process.
  /// If `true`, this action's output may be muted.
  /// If `false`, this action's output will never be muted.
  /// For most cases this field will be set automatically,
  /// so don't set this field unless you know what you are doing.
  pub maybe_muted: bool,

  possible_kinds: HashSet<Kind>,
  exec: Box<dyn Fn(&mut ActionInput<ActionState>) -> Option<ActionOutput<Kind, ErrorType>>>,
}

impl<Kind: 'static, ActionState: 'static, ErrorType: 'static> Action<Kind, ActionState, ErrorType> {
  pub fn possible_kinds(&self) -> &HashSet<Kind> {
    &self.possible_kinds
  }

  pub fn exec(
    &self,
    input: &mut ActionInput<ActionState>,
  ) -> Option<ActionOutput<Kind, ErrorType>> {
    (self.exec)(input)
  }

  pub fn apply<F>(self, decorator: F) -> Action<Kind, ActionState, ErrorType>
  where
    F: Fn(
        AcceptedActionDecoratorContext<Kind, ActionState, ErrorType>,
      ) -> Option<ActionOutput<Kind, ErrorType>>
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
}
