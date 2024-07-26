use super::Lexer;

/// With this struct you can continue a finished lex.
/// For most cases this will be constructed by [`ForkEnabled`](crate::lexer::fork::ForkEnabled)
/// (when lexing with [`LexOptions::fork`](crate::lexer::options::LexOptions::fork) enabled).
/// You can also construct this if you implement [`LexOptionsFork`](crate::lexer::fork::LexOptionsFork),
/// but make sure you know what you are doing.
#[derive(PartialEq, Clone, Debug)]
pub struct ReLexContext {
  /// See [`Self::skip`].
  pub start: usize,
  /// How many actions are skipped.
  /// This is effective only if
  /// the [`ActionInput::start`](crate::lexer::action::ActionInput::start)
  /// equals to [`Self::start`].
  pub skip: usize,
}

impl ReLexContext {
  #[inline]
  pub const fn new() -> Self {
    // set skip to 0 means this is not a re-lex
    Self { start: 0, skip: 0 }
  }
}

impl Default for ReLexContext {
  fn default() -> Self {
    Self::new()
  }
}

pub trait ReLexableFactory<'text, Kind: 'static, ActionState, ErrorType> {
  type StatelessReLexableType: Default;
  type ReLexableType;

  /// This is used to backup the action state as needed.
  fn before_mutate_action_state(&mut self, action_state: &ActionState);

  fn into_stateless_re_lexable(
    self,
    start: usize,
    actions_len: usize,
    action_index: usize,
  ) -> Self::StatelessReLexableType;

  /// This should be called before [`Lexer::state`] is mutated.
  fn into_re_lexable(
    stateless_re_lexable: Self::StatelessReLexableType,
    lexer: &Lexer<'text, Kind, ActionState, ErrorType>,
  ) -> Self::ReLexableType;
}

impl<'text, Kind: 'static, ActionState, ErrorType>
  ReLexableFactory<'text, Kind, ActionState, ErrorType> for ()
{
  type StatelessReLexableType = ();
  type ReLexableType = ();

  fn before_mutate_action_state(&mut self, _action_state: &ActionState) {}

  fn into_stateless_re_lexable(
    self,
    _start: usize,
    _actions_len: usize,
    _action_index: usize,
  ) -> Self::StatelessReLexableType {
    ()
  }

  fn into_re_lexable(
    _stateless_re_lexable: Self::StatelessReLexableType,
    _lexer: &Lexer<'text, Kind, ActionState, ErrorType>,
  ) -> Self::ReLexableType {
    ()
  }
}

pub struct ReLexableBuilder<ActionState> {
  /// The action state before any mutation in the current lex.
  action_state_bk: Option<ActionState>,
}

impl<ActionState> Default for ReLexableBuilder<ActionState> {
  fn default() -> Self {
    Self {
      action_state_bk: None,
    }
  }
}

impl<'text, Kind: 'static, ActionState: Clone, ErrorType>
  ReLexableFactory<'text, Kind, ActionState, ErrorType> for ReLexableBuilder<ActionState>
{
  type StatelessReLexableType = Option<(Option<ActionState>, ReLexContext)>;
  type ReLexableType = Option<(Lexer<'text, Kind, ActionState, ErrorType>, ReLexContext)>;

  fn before_mutate_action_state(&mut self, action_state: &ActionState) {
    // backup the action state before the first mutation during one lexing loop
    if self.action_state_bk.is_none() {
      self.action_state_bk = Some(action_state.clone());
    }
  }

  fn into_stateless_re_lexable(
    self,
    start: usize,
    actions_len: usize,
    action_index: usize,
  ) -> Self::StatelessReLexableType {
    if action_index < actions_len - 1 {
      // current action is not the last one
      // so the lex is re-lex-able
      Some((
        // the backup action state could be `None` to indicate no mutation happened
        self.action_state_bk,
        ReLexContext {
          skip: action_index + 1, // index + 1 is the count of actions to skip
          start,
        },
      ))
    } else {
      // current action is the last one
      // no next action to re-lex
      None
    }
  }

  fn into_re_lexable(
    stateless_re_lexable: Self::StatelessReLexableType,
    lexer: &Lexer<'text, Kind, ActionState, ErrorType>,
  ) -> Self::ReLexableType {
    stateless_re_lexable.map(|(action_state_bk, ctx)| {
      (
        action_state_bk
          // if there is a backup action state, it means the lexer's action state is mutated
          // so clone the lexer with the backup action state
          .map(|action_state_bk| lexer.clone_with(action_state_bk))
          // if there is no backup action state, it means the lexer's action state is not mutated
          // just clone the lexer
          .unwrap_or_else(|| lexer.clone()),
        ctx,
      )
    })
  }
}
