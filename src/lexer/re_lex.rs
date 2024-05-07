use super::{expectation::Expectation, state::LexerState};

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

impl Default for ReLexContext {
  fn default() -> Self {
    // set skip to 0 means this is not a re-lex
    Self { start: 0, skip: 0 }
  }
}

pub struct ReLexable<'text, 'expect_text, Kind: 'static, ActionState> {
  /// The re-lexable lex's expectation.
  pub expectation: Expectation<'expect_text, Kind>,
  /// If [`Some`], this will override [`Lexer::action_state`](crate::lexer::Lexer::action_state).
  /// This will be [`Some`] if the re-lexable lex mutated the action state.
  /// This will be ignored by [`StatelessLexer`](crate::lexer::stateless::StatelessLexer).
  pub action_state: Option<ActionState>,
  pub lexer_state: LexerState<'text>, // TODO: should this use LexerState or digested+text?
  pub ctx: ReLexContext,
}
