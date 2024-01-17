pub mod input;
pub mod output;
pub mod regex;
pub mod simple;

use self::{input::ActionInput, output::ActionOutput};
// use std::collections::HashSet;

// pub struct AcceptedActionDecoratorContext<'buffer, 'state, Kind, Data, ActionState, ErrorType> {
//   input: ActionInput<'buffer, 'state, ActionState>,
//   output: WrappedActionOutput<'buffer, Kind, Data, ErrorType>,
// }

pub struct Action<Kind, ActionState, ErrorType> {
  // possible_kinds: HashSet<Kind>,
  // maybe_muted: bool,
  exec: Box<dyn Fn(&mut ActionInput<ActionState>) -> Option<ActionOutput<Kind, ErrorType>>>,
}

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  pub fn exec(
    &self,
    input: &mut ActionInput<ActionState>,
  ) -> Option<ActionOutput<Kind, ErrorType>> {
    (self.exec)(input)
  }
}
