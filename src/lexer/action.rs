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
  // pub possible_kinds: HashSet<Kind>,
  // pub maybe_muted: bool,
  pub exec: Box<dyn Fn(&mut ActionInput<ActionState>) -> Option<ActionOutput<Kind, ErrorType>>>,
}
