use super::{instant::Instant, Lexer};

/// With this struct you can retry a lex with different actions.
///
/// This will be constructed by [`ForkEnabled`](crate::lexer::fork::ForkEnabled)
/// (when lexing or peeking, with [`LexOptions::fork`](crate::lexer::options::LexOptions::fork) enabled).
/// You should never construct this struct manually
/// because whe [`StatelessLexer`](crate::lexer::stateless::StatelessLexer) will skip
/// actions as needed and it is not guaranteed the memory layout of this struct are stable across versions.
/// # Caveats
/// Be careful with stateful lexers, because when actions are skipped your lexer's state
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
///
/// // the first lex will emit `>>`, which is not what we want
/// let output = lexer.lex_with(|o| o.fork());
/// assert_eq!(&text[output.token.unwrap().range], ">>");
///
/// // since we enabled `fork`, the lexer will return a re-lexable.
/// // we can try to transform the re-lexable into a lexer and a re-lex context
/// let (mut lexer, context) = output.re_lexable.into_lexer(&lexer).unwrap();
///
/// // lex with the re-lex context to retry the lex,
/// // but skip `exact(">>")` when lexing ">>"
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
pub trait ReLexableFactory<'text, Kind: 'static, State, ErrorType> {
  /// This should extends [`Default`] so when no token is emitted,
  /// the output can be created with a default value.
  type StatelessReLexableType: Default;
  type ReLexableType;

  /// This will be called only once before the first mutation of the action state.
  fn backup_state(&mut self, state: &State);

  fn into_stateless_re_lexable(
    self,
    start: usize,
    actions_len: usize,
    action_index: usize,
  ) -> Self::StatelessReLexableType;

  /// This should be called before [`Lexer::state`] is mutated
  /// to ensure the re-lexable has the state before the mutation.
  fn build_re_lexable(
    stateless_re_lexable: Self::StatelessReLexableType,
    digested: usize,
    lexer: &Lexer<'text, Kind, State, ErrorType>,
  ) -> Self::ReLexableType;
}

// mock re-lexable factory
impl<'text, Kind: 'static, State, ErrorType> ReLexableFactory<'text, Kind, State, ErrorType>
  for ()
{
  type StatelessReLexableType = ();
  type ReLexableType = ();

  #[inline]
  fn backup_state(&mut self, _state: &State) {}

  #[inline]
  fn into_stateless_re_lexable(
    self,
    _start: usize,
    _actions_len: usize,
    _action_index: usize,
  ) -> Self::StatelessReLexableType {
  }

  #[inline]
  fn build_re_lexable(
    _stateless_re_lexable: Self::StatelessReLexableType,
    _digested: usize,
    _lexer: &Lexer<'text, Kind, State, ErrorType>,
  ) -> Self::ReLexableType {
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatelessReLexable<State> {
  /// The action state before any mutation in the current lex.
  /// If [`None`], it means no mutation happened.
  ///
  /// This will always be [`None`] when peeking
  /// because the original state is not mutated.
  pub state_bk: Option<State>, // users can always mutate the action state directly so it is ok to expose it
  /// If [`Some`], it means the lex is re-lexable.
  pub ctx: Option<ReLexContext>, // ReLexContext's fields are private so its ok to expose it
}

impl<State> Default for StatelessReLexable<State> {
  #[inline]
  fn default() -> Self {
    Self {
      state_bk: None,
      ctx: None,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReLexable<'text, State> {
  pub stateless: StatelessReLexable<State>,
  /// The backup-ed lexer state before it is mutated.
  /// If [`None`], it means the current lex digest 0 bytes.
  /// This is private to prevent caller from mutating the lexer state directly.
  state_bk: Option<Instant<'text>>,
  // TODO: add a ref of the lexer as a guard to prevent caller from mutating the lexer
  // before applying this re-lexable?
}

impl<'text, State: Clone> ReLexable<'text, State> {
  /// Consume self, try to build a lexer with the state before previous lexing.
  /// Return [`None`] if the lex is not re-lexable.
  ///
  /// See [`ReLexContext`] for more details.
  pub fn into_lexer<Kind, ErrorType>(
    self,
    lexer: &Lexer<'text, Kind, State, ErrorType>,
  ) -> Option<(Lexer<'text, Kind, State, ErrorType>, ReLexContext)> {
    self.stateless.ctx.map(|ctx| {
      (
        Lexer::from_re_lexable(
          lexer.stateless().clone(),
          self
            .stateless
            .state_bk
            .unwrap_or_else(|| lexer.state.clone()),
          self.state_bk.unwrap_or_else(|| lexer.state().clone()),
        ),
        ctx,
      )
    })
  }
}

pub struct ReLexableBuilder<State> {
  /// See [`StatelessReLexable::state_bk`].
  state_bk: Option<State>,
}

impl<State> Default for ReLexableBuilder<State> {
  #[inline]
  fn default() -> Self {
    Self { state_bk: None }
  }
}

impl<'text, Kind: 'static, State: Clone, ErrorType> ReLexableFactory<'text, Kind, State, ErrorType>
  for ReLexableBuilder<State>
{
  type StatelessReLexableType = StatelessReLexable<State>;
  type ReLexableType = ReLexable<'text, State>;

  fn backup_state(&mut self, state: &State) {
    // this should only be called once to prevent duplicated clone of the action state,
    // so the action state backup must be none
    debug_assert!(
      self.state_bk.is_none(),
      "action state backup is already set"
    );

    // backup the action state before the first mutation during one lexing loop
    self.state_bk = Some(state.clone());
  }

  fn into_stateless_re_lexable(
    self,
    start: usize,
    actions_len: usize,
    action_index: usize,
  ) -> Self::StatelessReLexableType {
    Self::StatelessReLexableType {
      state_bk: self.state_bk,
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

  fn build_re_lexable(
    stateless_re_lexable: Self::StatelessReLexableType,
    digested: usize,
    lexer: &Lexer<'text, Kind, State, ErrorType>,
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
    let state = 0;
    ReLexableFactory::<(), _, ()>::backup_state(&mut factory, &state);
    let stateless_re_lexable =
      ReLexableFactory::<(), i32, ()>::into_stateless_re_lexable(factory, 0, 2, 1);
    assert_eq!(stateless_re_lexable, ());
    let lexer = LexerBuilder::<()>::new().build("");
    let re_lexable =
      <() as ReLexableFactory<_, _, _>>::build_re_lexable(stateless_re_lexable, 0, &lexer);
    assert_eq!(re_lexable, ());
  }

  #[test]
  fn test_stateless_re_lexable() {
    let builder = StatelessReLexable::<()>::default();
    assert_eq!(
      builder,
      StatelessReLexable {
        state_bk: None,
        ctx: None
      }
    );
  }

  #[test]
  fn test_re_lexable() {
    let re_lexable = ReLexable {
      stateless: StatelessReLexable {
        state_bk: Some(1),
        ctx: Some(ReLexContext { start: 1, skip: 1 }),
      },
      state_bk: {
        let mut s = Instant::new("123");
        s.digest(1);
        s
      }
      .into(),
    };
    let (lexer, ctx) = re_lexable
      .into_lexer(&LexerBuilder::<()>::stateful().build(""))
      .unwrap();
    assert_eq!(ctx, ReLexContext { start: 1, skip: 1 });
    assert_eq!(lexer.state().digested(), 1);
    assert_eq!(lexer.state, 1);
  }

  #[test]
  fn re_lexable_builder() {
    let mut builder = ReLexableBuilder::default();
    let state = 0;
    ReLexableFactory::<(), _, ()>::backup_state(&mut builder, &state);
    let stateless_re_lexable =
      ReLexableFactory::<(), i32, ()>::into_stateless_re_lexable(builder, 0, 2, 1);
    assert_eq!(
      stateless_re_lexable,
      StatelessReLexable {
        state_bk: Some(0),
        ctx: None
      }
    );
    let lexer = LexerBuilder::<(), _>::stateful().build("");
    let re_lexable = ReLexableBuilder::build_re_lexable(stateless_re_lexable.clone(), 0, &lexer);
    assert_eq!(
      re_lexable,
      ReLexable {
        stateless: stateless_re_lexable.clone(),
        state_bk: None
      }
    );
    let re_lexable = ReLexableBuilder::build_re_lexable(stateless_re_lexable.clone(), 1, &lexer);
    assert_eq!(
      re_lexable,
      ReLexable {
        stateless: stateless_re_lexable,
        state_bk: Some(lexer.state().clone())
      }
    );

    let mut builder = ReLexableBuilder::default();
    let state = 0;
    ReLexableFactory::<(), _, ()>::backup_state(&mut builder, &state);
    let stateless_re_lexable =
      ReLexableFactory::<(), i32, ()>::into_stateless_re_lexable(builder, 0, 2, 0);
    assert_eq!(
      stateless_re_lexable,
      StatelessReLexable {
        state_bk: Some(0),
        ctx: Some(ReLexContext { start: 0, skip: 1 })
      }
    );
    let mut lexer = LexerBuilder::<(), _>::stateful().build("");
    lexer.state = 1;
    let re_lexable = ReLexableBuilder::build_re_lexable(stateless_re_lexable.clone(), 0, &lexer);
    assert_eq!(
      re_lexable,
      ReLexable {
        stateless: stateless_re_lexable.clone(),
        state_bk: None
      }
    );
    let re_lexable = ReLexableBuilder::build_re_lexable(stateless_re_lexable.clone(), 1, &lexer);
    assert_eq!(
      re_lexable,
      ReLexable {
        stateless: stateless_re_lexable,
        state_bk: Some(lexer.state().clone())
      }
    );
  }

  #[test]
  #[should_panic]
  fn re_lexable_builder_multi_call_to_mutate_state() {
    let mut builder = ReLexableBuilder::default();
    let state = 0;
    ReLexableFactory::<(), _, ()>::backup_state(&mut builder, &state);
    ReLexableFactory::<(), _, ()>::backup_state(&mut builder, &state);
  }
}
