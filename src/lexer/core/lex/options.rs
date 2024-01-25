use super::expectation::Expectation;

pub struct LexerCoreLexOptions<'action_state, 'expect, Kind, ActionState: Clone + Default> {
  pub start: usize,
  pub action_state: &'action_state mut ActionState,
  pub expectation: Expectation<'expect, Kind>,
}
