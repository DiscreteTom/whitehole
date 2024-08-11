use super::{instant::Instant, re_lex::ReLexContext, Lexer};

/// See [`LexOptions::fork`](crate::lexer::options::LexOptions::fork).
///
/// These types already implement the [`LexOptionsFork`] trait:
/// - `()` - means the fork feature is disabled.
/// - [`ForkEnabled`] - means the fork feature is enabled.
///
/// We use this trait instead of a [`bool`] value
/// to implement the [`fork`](crate::lexer::options::LexOptions::fork) feature
/// so that we can return different types in [`ForkOutputFactory::build_fork_output`]
/// to avoid unnecessary allocations.
pub trait LexOptionsFork<'text, Kind, State, ErrorType> {
  // this has to implement `Default` because the instance is not provided by the user
  // and we have to create the instance by our own
  type OutputFactoryType: ForkOutputFactory<'text, Kind, State, ErrorType> + Default;
}

// the mock implementation of the fork feature
impl<'text, Kind, State, ErrorType> LexOptionsFork<'text, Kind, State, ErrorType> for () {
  type OutputFactoryType = ();
}

/// This struct is used to indicate that the fork feature is enabled.
/// This struct implements [`LexOptionsFork`].
/// See [`LexOptions::fork`](crate::lexer::options::LexOptions::fork).
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct ForkEnabled;

impl<'text, Kind, State: Clone, ErrorType> LexOptionsFork<'text, Kind, State, ErrorType>
  for ForkEnabled
{
  type OutputFactoryType = ForkOutputBuilder<State>;
}

/// These types already implement the [`ForkOutputFactory`] trait:
/// - `()` - no re-lexable will be created.
/// - [`ForkOutputBuilder`] - create re-lexable structs if possible.
pub trait ForkOutputFactory<'text, Kind, State, ErrorType> {
  /// This should extends [`Default`] so when no token is emitted,
  /// the output can be created with a default value.
  type StatelessForkOutputType: Default;
  type ForkOutputType;

  /// This will be called only once before the first mutation of the action state.
  fn backup_state(&mut self, state: &State);

  fn into_stateless_fork_output(
    self,
    start: usize,
    actions_len: usize,
    action_index: usize,
  ) -> Self::StatelessForkOutputType;

  /// This should be called before [`Lexer::state`] is mutated
  /// to ensure the re-lexable has the state before the mutation.
  fn build_fork_output(
    stateless_re_lexable: Self::StatelessForkOutputType,
    digested: usize,
    lexer: &Lexer<'text, Kind, State, ErrorType>,
  ) -> Self::ForkOutputType;
}

// mock re-lexable factory
impl<'text, Kind, State, ErrorType> ForkOutputFactory<'text, Kind, State, ErrorType> for () {
  type StatelessForkOutputType = ();
  type ForkOutputType = ();

  #[inline]
  fn backup_state(&mut self, _state: &State) {}

  #[inline]
  fn into_stateless_fork_output(
    self,
    _start: usize,
    _actions_len: usize,
    _action_index: usize,
  ) -> Self::StatelessForkOutputType {
  }

  #[inline]
  fn build_fork_output(
    _stateless_re_lexable: Self::StatelessForkOutputType,
    _digested: usize,
    _lexer: &Lexer<'text, Kind, State, ErrorType>,
  ) -> Self::ForkOutputType {
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatelessForkOutput<State> {
  /// The action state before any mutation in the current lex.
  /// If [`None`], it means no mutation happened.
  ///
  /// This will always be [`None`] when peeking
  /// because the original state is not mutated.
  pub state: Option<State>, // users can always mutate the action state directly so it is ok to expose it
  /// If [`Some`], it means the lex is re-lexable.
  pub ctx: Option<ReLexContext>, // ReLexContext's fields are private so its ok to expose it
}

impl<State> Default for StatelessForkOutput<State> {
  #[inline]
  fn default() -> Self {
    Self {
      state: None,
      ctx: None,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForkOutput<'text, State> {
  pub stateless: StatelessForkOutput<State>,
  /// The backup-ed lexer state before it is mutated.
  /// If [`None`], it means the current lex digest 0 bytes.
  /// This is private to prevent caller from mutating the lexer state directly.
  instant: Option<Instant<'text>>,
  // TODO: add a ref of the lexer as a guard to prevent caller from mutating the lexer
  // before applying this re-lexable?
}

impl<'text, State: Clone> ForkOutput<'text, State> {
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
          self.stateless.state.unwrap_or_else(|| lexer.state.clone()),
          self.instant.unwrap_or_else(|| lexer.instant().clone()),
        ),
        ctx,
      )
    })
  }
}

pub struct ForkOutputBuilder<State> {
  /// See [`StatelessForkOutput::state`].
  state: Option<State>,
}

impl<State> Default for ForkOutputBuilder<State> {
  #[inline]
  fn default() -> Self {
    Self { state: None }
  }
}

impl<'text, Kind, State: Clone, ErrorType> ForkOutputFactory<'text, Kind, State, ErrorType>
  for ForkOutputBuilder<State>
{
  type StatelessForkOutputType = StatelessForkOutput<State>;
  type ForkOutputType = ForkOutput<'text, State>;

  fn backup_state(&mut self, state: &State) {
    // this should only be called once to prevent duplicated clone of the action state,
    // so the action state backup must be none
    debug_assert!(self.state.is_none(), "action state backup is already set");

    // backup the action state before the first mutation during one lexing loop
    self.state = Some(state.clone());
  }

  fn into_stateless_fork_output(
    self,
    start: usize,
    actions_len: usize,
    action_index: usize,
  ) -> Self::StatelessForkOutputType {
    Self::StatelessForkOutputType {
      state: self.state,
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

  fn build_fork_output(
    stateless_re_lexable: Self::StatelessForkOutputType,
    digested: usize,
    lexer: &Lexer<'text, Kind, State, ErrorType>,
  ) -> Self::ForkOutputType {
    Self::ForkOutputType {
      stateless: stateless_re_lexable,
      instant: (digested != 0).then(|| lexer.instant().clone()),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::LexerBuilder;

  #[test]
  fn mock_re_lexable_factory() {
    let mut factory = ();
    let state = 0;
    ForkOutputFactory::<(), _, ()>::backup_state(&mut factory, &state);
    let stateless_re_lexable =
      ForkOutputFactory::<(), i32, ()>::into_stateless_fork_output(factory, 0, 2, 1);
    assert_eq!(stateless_re_lexable, ());
    let lexer = LexerBuilder::<()>::new().build("");
    let re_lexable =
      <() as ForkOutputFactory<_, _, _>>::build_fork_output(stateless_re_lexable, 0, &lexer);
    assert_eq!(re_lexable, ());
  }

  #[test]
  fn test_stateless_re_lexable() {
    let builder = StatelessForkOutput::<()>::default();
    assert_eq!(
      builder,
      StatelessForkOutput {
        state: None,
        ctx: None
      }
    );
  }

  #[test]
  fn test_re_lexable() {
    let re_lexable = ForkOutput {
      stateless: StatelessForkOutput {
        state: Some(1),
        ctx: Some(ReLexContext { start: 1, skip: 1 }),
      },
      instant: {
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
    assert_eq!(lexer.instant().digested(), 1);
    assert_eq!(lexer.state, 1);
  }

  #[test]
  fn re_lexable_builder() {
    let mut builder = ForkOutputBuilder::default();
    let state = 0;
    ForkOutputFactory::<(), _, ()>::backup_state(&mut builder, &state);
    let stateless_re_lexable =
      ForkOutputFactory::<(), i32, ()>::into_stateless_fork_output(builder, 0, 2, 1);
    assert_eq!(
      stateless_re_lexable,
      StatelessForkOutput {
        state: Some(0),
        ctx: None
      }
    );
    let lexer = LexerBuilder::<(), _>::stateful().build("");
    let re_lexable = ForkOutputBuilder::build_fork_output(stateless_re_lexable.clone(), 0, &lexer);
    assert_eq!(
      re_lexable,
      ForkOutput {
        stateless: stateless_re_lexable.clone(),
        instant: None
      }
    );
    let re_lexable = ForkOutputBuilder::build_fork_output(stateless_re_lexable.clone(), 1, &lexer);
    assert_eq!(
      re_lexable,
      ForkOutput {
        stateless: stateless_re_lexable,
        instant: Some(lexer.instant().clone())
      }
    );

    let mut builder = ForkOutputBuilder::default();
    let state = 0;
    ForkOutputFactory::<(), _, ()>::backup_state(&mut builder, &state);
    let stateless_re_lexable =
      ForkOutputFactory::<(), i32, ()>::into_stateless_fork_output(builder, 0, 2, 0);
    assert_eq!(
      stateless_re_lexable,
      StatelessForkOutput {
        state: Some(0),
        ctx: Some(ReLexContext { start: 0, skip: 1 })
      }
    );
    let mut lexer = LexerBuilder::<(), _>::stateful().build("");
    lexer.state = 1;
    let re_lexable = ForkOutputBuilder::build_fork_output(stateless_re_lexable.clone(), 0, &lexer);
    assert_eq!(
      re_lexable,
      ForkOutput {
        stateless: stateless_re_lexable.clone(),
        instant: None
      }
    );
    let re_lexable = ForkOutputBuilder::build_fork_output(stateless_re_lexable.clone(), 1, &lexer);
    assert_eq!(
      re_lexable,
      ForkOutput {
        stateless: stateless_re_lexable,
        instant: Some(lexer.instant().clone())
      }
    );
  }

  #[test]
  #[should_panic]
  fn re_lexable_builder_multi_call_to_mutate_state() {
    let mut builder = ForkOutputBuilder::default();
    let state = 0;
    ForkOutputFactory::<(), _, ()>::backup_state(&mut builder, &state);
    ForkOutputFactory::<(), _, ()>::backup_state(&mut builder, &state);
  }
}
