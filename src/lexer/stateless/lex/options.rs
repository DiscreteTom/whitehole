use super::expectation::Expectation;

// TODO: move
pub struct StatelessLexOptions<'action_state, 'expect, Kind, ActionState: Clone + Default> {
  pub start: usize,
  pub action_state: &'action_state mut ActionState,
  pub expectation: Expectation<'expect, Kind>,
}
