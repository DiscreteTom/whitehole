pub mod input;
pub mod output;
pub mod regex;
pub mod simple;

use self::{input::ActionInput, output::ActionOutput};

pub trait Action<Kind, ActionState> {
  fn exec(&self, input: &'static ActionInput<ActionState>) -> ActionOutput<Kind>;
}
