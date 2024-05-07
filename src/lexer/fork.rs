use super::re_lex::ReLexContext;

/// See [`LexOptions::fork`](crate::lexer::options::LexOptions::fork).
// we use this trait and 2 structs instead of a `bool` to implement the `Fork` feature
// so that we can return different types in `into_re_lexable` to avoid unnecessary allocations
pub trait LexOptionsFork<'text, 'expect_text, Kind: 'static, ActionState>: Default {
  type ReLexableType: Default;

  fn before_mutate_action_state(&mut self, action_state: &ActionState);
  fn into_re_lexable(
    self,
    start: usize,
    actions_len: usize,
    action_index: usize,
  ) -> Self::ReLexableType;
}
pub struct ForkEnabled<ActionState: Clone> {
  /// The action state before any mutation in the current lex.
  action_state_bk: Option<ActionState>, // TODO: store this in another struct
}
impl<ActionState: Clone> Default for ForkEnabled<ActionState> {
  fn default() -> Self {
    Self {
      action_state_bk: None,
    }
  }
}
impl<'text, 'expect_text, Kind: 'static, ActionState: Clone>
  LexOptionsFork<'text, 'expect_text, Kind, ActionState> for ForkEnabled<ActionState>
{
  type ReLexableType = Option<(ReLexContext, Option<ActionState>)>;

  fn before_mutate_action_state(&mut self, action_state: &ActionState) {
    if self.action_state_bk.is_none() {
      self.action_state_bk = Some(action_state.clone());
    }
  }

  fn into_re_lexable(
    self,
    start: usize,
    actions_len: usize,
    action_index: usize,
  ) -> Self::ReLexableType {
    if action_index < actions_len - 1 {
      // current action is not the last one
      // so the lex is re-lex-able
      Some((
        ReLexContext {
          skip: action_index + 1, // index + 1 is the count of actions to skip
          start,
        },
        self.action_state_bk,
      ))
    } else {
      // current action is the last one
      // no next action to re-lex
      None
    }
  }
}
#[derive(Default)]
pub struct ForkDisabled;
impl<'text, 'expect_text, Kind: 'static, ActionState>
  LexOptionsFork<'text, 'expect_text, Kind, ActionState> for ForkDisabled
{
  type ReLexableType = ();

  fn before_mutate_action_state(&mut self, _action_state: &ActionState) {}

  fn into_re_lexable(
    self,
    _start: usize,
    _actions_len: usize,
    _action_index: usize,
  ) -> Self::ReLexableType {
    ()
  }
}
