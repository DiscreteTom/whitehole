use super::Lexer;

/// With this struct you can retry a lex with different actions.
///
/// This will be constructed by [`ForkEnabled`](crate::lexer::fork::ForkEnabled)
/// (when lexing with [`LexOptions::fork`](crate::lexer::options::LexOptions::fork) enabled).
/// You should never construct this struct manually
/// because whe [`StatelessLexer`](crate::lexer::stateless::StatelessLexer) will skip
/// actions as needed and it is not guaranteed the fields of this struct are stable across versions.
/// # Caveats
/// Be careful with stateful lexers, because when actions are skipped your action state
/// may be inconsistent with the original lexing.
/// # Examples
/// ```
/// # use whitehole::lexer::{action::{exact, regex}, LexerBuilder};
/// let text = "Option<Option<()>>";
/// let mut lexer = LexerBuilder::new()
///   // try to match `>>` first, if failed, try to match `>`
///   .append([exact(">>"), exact(">")])
///   // ignore all other characters
///   .ignore(regex(".").unchecked_head_unknown())
///   .build(text);
/// // the first lex will emit `>>`, which is not what we want
/// let output = lexer.lex_with(|o| o.fork());
/// assert_eq!(&text[output.token.unwrap().range], ">>");
/// // since we enabled `fork`, the lexer will return a re-lexable if possible.
/// let (mut lexer, context) = output.re_lexable.unwrap();
/// // lex with the re-lex context to retry the lex, but skip `exact(">>")` when lexing ">>"
/// let output = lexer.lex_with(|o| o.re_lex(context));
/// // now the lexer will emit `>`
/// assert_eq!(&text[output.token.unwrap().range], ">");
/// ```
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct ReLexContext {
  /// See [`Self::skip`].
  pub(crate) start: usize,
  /// How many actions are skipped.
  /// This is effective only if
  /// the [`ActionInput::start`](crate::lexer::action::ActionInput::start)
  /// equals to [`Self::start`].
  pub(crate) skip: usize,
}

impl ReLexContext {
  /// Create a new re-lex context with re-lex disabled.
  #[inline]
  pub const fn new() -> Self {
    // set skip to 0 means this is not a re-lex
    Self { start: 0, skip: 0 }
  }
}

/// These types already implement the [`ReLexableFactory`] trait:
/// - `()` - no re-lexable will be created.
/// - [`ReLexableBuilder`] - create re-lexable structs if possible.
pub trait ReLexableFactory<'text, Kind: 'static, ActionState, ErrorType> {
  /// This should extends [`Default`] so when no token is emitted,
  /// the output can be created with a default value.
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

  /// This should be called before [`Lexer::state`] is mutated
  /// to ensure the re-lexable has the state before the mutation.
  fn into_re_lexable(
    stateless_re_lexable: Self::StatelessReLexableType,
    lexer: &Lexer<'text, Kind, ActionState, ErrorType>,
  ) -> Self::ReLexableType;
}

// mock re-lexable factory
impl<'text, Kind: 'static, ActionState, ErrorType>
  ReLexableFactory<'text, Kind, ActionState, ErrorType> for ()
{
  type StatelessReLexableType = ();
  type ReLexableType = ();

  #[inline]
  fn before_mutate_action_state(&mut self, _action_state: &ActionState) {}

  #[inline]
  fn into_stateless_re_lexable(
    self,
    _start: usize,
    _actions_len: usize,
    _action_index: usize,
  ) -> Self::StatelessReLexableType {
  }

  #[inline]
  fn into_re_lexable(
    _stateless_re_lexable: Self::StatelessReLexableType,
    _lexer: &Lexer<'text, Kind, ActionState, ErrorType>,
  ) -> Self::ReLexableType {
  }
}

pub struct ReLexableBuilder<ActionState> {
  /// The action state before any mutation in the current lex.
  action_state_bk: Option<ActionState>,
}

impl<ActionState> Default for ReLexableBuilder<ActionState> {
  #[inline]
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
          // to get a lexer with the state before the mutation
          .map(|action_state_bk| lexer.clone_with(action_state_bk))
          // if there is no backup action state, it means the lexer's action state is not mutated
          // just clone the lexer
          .unwrap_or_else(|| lexer.clone()),
        ctx,
      )
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::LexerBuilder;

  #[test]
  fn re_lex_context() {
    let context = ReLexContext::new();
    assert_eq!(context, ReLexContext { start: 0, skip: 0 });
    let context = ReLexContext::default();
    assert_eq!(context, ReLexContext { start: 0, skip: 0 });
  }

  #[test]
  fn mock_re_lexable_factory() {
    let mut factory = ();
    let action_state = 0;
    ReLexableFactory::<(), _, ()>::before_mutate_action_state(&mut factory, &action_state);
    let stateless_re_lexable =
      ReLexableFactory::<(), i32, ()>::into_stateless_re_lexable(factory, 0, 2, 1);
    assert_eq!(stateless_re_lexable, ());
    let lexer = LexerBuilder::<()>::new().build("");
    let re_lexable =
      <() as ReLexableFactory<_, _, _>>::into_re_lexable(stateless_re_lexable, &lexer);
    assert_eq!(re_lexable, ());
  }

  #[test]
  fn re_lexable_builder() {
    let mut builder = ReLexableBuilder::default();
    let action_state = 0;
    ReLexableFactory::<(), _, ()>::before_mutate_action_state(&mut builder, &action_state);
    let stateless_re_lexable =
      ReLexableFactory::<(), i32, ()>::into_stateless_re_lexable(builder, 0, 2, 1);
    assert_eq!(stateless_re_lexable, None);
    let lexer = LexerBuilder::<(), _>::stateful().build("");
    let re_lexable = ReLexableBuilder::into_re_lexable(stateless_re_lexable, &lexer);
    assert!(re_lexable.is_none());

    let mut builder = ReLexableBuilder::default();
    let action_state = 0;
    ReLexableFactory::<(), _, ()>::before_mutate_action_state(&mut builder, &action_state);
    let stateless_re_lexable =
      ReLexableFactory::<(), i32, ()>::into_stateless_re_lexable(builder, 0, 2, 0);
    assert_eq!(
      stateless_re_lexable,
      Some((Some(0), ReLexContext { start: 0, skip: 1 }))
    );
    let mut lexer = LexerBuilder::<(), _>::stateful().build("");
    lexer.action_state = 1;
    let (lexer, context) = ReLexableBuilder::into_re_lexable(stateless_re_lexable, &lexer).unwrap();
    assert_eq!(context, ReLexContext { start: 0, skip: 1 });
    assert_eq!(lexer.action_state, 0);
  }
}
