use super::{state::LexerState, Lexer};

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
/// // since we enabled `fork`, the lexer will return a re-lexable.
/// // we can transform the re-lexable into a lexer and a re-lex context
/// let (mut lexer, context) = output.re_lexable.into_lexer(&lexer).unwrap();
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
    digested: usize,
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
    _digested: usize,
    _lexer: &Lexer<'text, Kind, ActionState, ErrorType>,
  ) -> Self::ReLexableType {
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatelessReLexable<ActionState> {
  /// The action state before any mutation in the current lex.
  /// If [`None`], it means no mutation happened.
  ///
  /// Even if the lexing is not re-lexable ([`Self::ctx`] is [`None`]),
  /// the backup-ed action state may be useful by the caller,
  /// e.g. peeking is a special case of fork, which needs to backup the action state before mutation,
  /// but doesn't need the re-lex context.
  pub action_state_bk: Option<ActionState>, // users can always mutate the action state directly so it is ok to expose it
  /// If [`Some`], it means the lex is re-lexable.
  pub ctx: Option<ReLexContext>, // ReLexContext's fields are private so its ok to expose it
}

impl<ActionState> Default for StatelessReLexable<ActionState> {
  #[inline]
  fn default() -> Self {
    Self {
      action_state_bk: None,
      ctx: None,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReLexable<'text, ActionState> {
  pub stateless: StatelessReLexable<ActionState>,
  /// The backup-ed lexer state before it is mutated.
  /// If [`None`], it means the current lex digest 0 bytes.
  /// This is private to prevent caller from mutating the lexer state directly.
  state_bk: Option<LexerState<'text>>,
  // TODO: add a ref of the lexer as a guard to prevent caller from mutating the lexer
  // before applying this re-lexable?
}

impl<'text, ActionState: Clone> ReLexable<'text, ActionState> {
  /// Consume self, try to build a lexer with the state before previous lexing.
  pub fn into_lexer<Kind, ErrorType>(
    self,
    lexer: &Lexer<'text, Kind, ActionState, ErrorType>,
  ) -> Option<(Lexer<'text, Kind, ActionState, ErrorType>, ReLexContext)> {
    self.stateless.ctx.map(|ctx| {
      (
        Lexer::from_re_lexable(
          lexer.stateless().clone(),
          self
            .stateless
            .action_state_bk
            .unwrap_or_else(|| lexer.action_state.clone()),
          self.state_bk.unwrap_or_else(|| lexer.state().clone()),
        ),
        ctx,
      )
    })
  }
}

pub struct ReLexableBuilder<ActionState> {
  /// See [`StatelessReLexable::action_state_bk`].
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
  type StatelessReLexableType = StatelessReLexable<ActionState>;
  type ReLexableType = ReLexable<'text, ActionState>;

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
    Self::StatelessReLexableType {
      action_state_bk: self.action_state_bk,
      ctx: if action_index < actions_len - 1 {
        // current action is not the last one
        // so the lex is re-lex-able
        Some(ReLexContext {
          skip: action_index + 1, // index + 1 is the count of actions to skip
          start,
        })
      } else {
        // current action is the last one
        // no next action to re-lex
        None
      },
    }
  }

  fn into_re_lexable(
    stateless_re_lexable: Self::StatelessReLexableType,
    digested: usize,
    lexer: &Lexer<'text, Kind, ActionState, ErrorType>,
  ) -> Self::ReLexableType {
    Self::ReLexableType {
      stateless: stateless_re_lexable,
      state_bk: (digested != 0).then(|| lexer.state().clone()),
    }
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
      <() as ReLexableFactory<_, _, _>>::into_re_lexable(stateless_re_lexable, 0, &lexer);
    assert_eq!(re_lexable, ());
  }

  #[test]
  fn re_lexable_builder() {
    let mut builder = ReLexableBuilder::default();
    let action_state = 0;
    ReLexableFactory::<(), _, ()>::before_mutate_action_state(&mut builder, &action_state);
    let stateless_re_lexable =
      ReLexableFactory::<(), i32, ()>::into_stateless_re_lexable(builder, 0, 2, 1);
    assert_eq!(
      stateless_re_lexable,
      StatelessReLexable {
        action_state_bk: Some(0),
        ctx: None
      }
    );
    let lexer = LexerBuilder::<(), _>::stateful().build("");
    let re_lexable = ReLexableBuilder::into_re_lexable(stateless_re_lexable.clone(), 0, &lexer);
    assert_eq!(
      re_lexable,
      ReLexable {
        stateless: stateless_re_lexable.clone(),
        state_bk: None
      }
    );
    let re_lexable = ReLexableBuilder::into_re_lexable(stateless_re_lexable.clone(), 1, &lexer);
    assert_eq!(
      re_lexable,
      ReLexable {
        stateless: stateless_re_lexable,
        state_bk: Some(lexer.state().clone())
      }
    );

    let mut builder = ReLexableBuilder::default();
    let action_state = 0;
    ReLexableFactory::<(), _, ()>::before_mutate_action_state(&mut builder, &action_state);
    let stateless_re_lexable =
      ReLexableFactory::<(), i32, ()>::into_stateless_re_lexable(builder, 0, 2, 0);
    assert_eq!(
      stateless_re_lexable,
      StatelessReLexable {
        action_state_bk: Some(0),
        ctx: Some(ReLexContext { start: 0, skip: 1 })
      }
    );
    let mut lexer = LexerBuilder::<(), _>::stateful().build("");
    lexer.action_state = 1;
    let re_lexable = ReLexableBuilder::into_re_lexable(stateless_re_lexable.clone(), 0, &lexer);
    assert_eq!(
      re_lexable,
      ReLexable {
        stateless: stateless_re_lexable.clone(),
        state_bk: None
      }
    );
    let re_lexable = ReLexableBuilder::into_re_lexable(stateless_re_lexable.clone(), 1, &lexer);
    assert_eq!(
      re_lexable,
      ReLexable {
        stateless: stateless_re_lexable,
        state_bk: Some(lexer.state().clone())
      }
    );
  }
}
