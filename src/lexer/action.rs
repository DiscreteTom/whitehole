pub mod input;
pub mod output;
pub mod regex;

use self::{input::ActionInput, output::ActionOutput};

pub trait Action<'action, Kind, ActionState> {
  fn exec(&self, input: &'action ActionInput<ActionState>) -> ActionOutput<Kind>;
}
