pub mod decorator;
pub mod input;
pub mod output;
pub mod regex;
pub mod select;
pub mod simple;

use self::{input::ActionInput, output::ActionOutput};
use super::token::TokenKindId;
use std::collections::HashSet;

pub struct Action<Kind, ActionState, ErrorType> {
  /// This flag is to indicate whether this action's output might be muted.
  /// The lexer will based on this flag to accelerate the lexing process.
  /// If `true`, this action's output may be muted.
  /// If `false`, this action's output will never be muted.
  /// For most cases this field will be set automatically,
  /// so don't set this field unless you know what you are doing.
  pub maybe_muted: bool,

  possible_kinds: HashSet<TokenKindId>,
  exec: Box<dyn Fn(&mut ActionInput<ActionState>) -> Option<ActionOutput<Kind, ErrorType>>>,
}

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  pub fn new<F>(exec: F) -> Action<(), ActionState, ErrorType>
  where
    F: Fn(&mut ActionInput<ActionState>) -> Option<ActionOutput<(), ErrorType>> + 'static,
  {
    Action {
      maybe_muted: false,
      possible_kinds: HashSet::new(),
      exec: Box::new(exec),
    }
  }

  pub fn never_muted(&self) -> bool {
    !self.maybe_muted
  }

  pub fn possible_kinds(&self) -> &HashSet<TokenKindId> {
    &self.possible_kinds
  }

  pub fn exec(
    &self,
    input: &mut ActionInput<ActionState>,
  ) -> Option<ActionOutput<Kind, ErrorType>> {
    (self.exec)(input)
  }
}
