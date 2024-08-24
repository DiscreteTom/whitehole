use super::{re_lex::ReLexContext, snapshot::PartialSnapshot, Lexer};

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
/// - `()` - no fork output will be created.
/// - [`ForkOutputBuilder`] - create fork output structs if possible.
pub trait ForkOutputFactory<'text, Kind, State, ErrorType> {
  /// This should extends [`Default`] so when no token is emitted,
  /// the output can be created with a default value.
  type StatelessForkOutputType: Default;
  type ForkOutputType;

  fn into_stateless_fork_output(
    self,
    start: usize,
    actions_len: usize,
    action_index: usize,
  ) -> Self::StatelessForkOutputType;

  /// This should be called before [`Lexer::instant`] is mutated
  /// to ensure the fork output has the instant before the mutation.
  fn build_fork_output(
    stateless: Self::StatelessForkOutputType,
    digested: usize,
    lexer: &Lexer<'text, Kind, State, ErrorType>,
  ) -> Self::ForkOutputType;
}

// mock fork output factory
impl<'text, Kind, State, ErrorType> ForkOutputFactory<'text, Kind, State, ErrorType> for () {
  type StatelessForkOutputType = ();
  type ForkOutputType = ();

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
    _stateless: Self::StatelessForkOutputType,
    _digested: usize,
    _lexer: &Lexer<'text, Kind, State, ErrorType>,
  ) -> Self::ForkOutputType {
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatelessForkOutput<State> {
  /// The state before any mutation in the current lex.
  /// If [`None`], it means no mutation happened.
  ///
  /// This will always be [`None`] when peeking
  /// because the original state is not mutated.
  pub state: Option<State>, // users can always mutate the state directly so it is ok to expose it
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
  /// If [`Some`], it means the lex is re-lexable.
  pub ctx: Option<ReLexContext>, // ReLexContext's fields are private so its ok to expose it
  /// The snapshot of the lexer before the lex.
  /// - If [`PartialSnapshot::state`] is [`None`], it means no mutation happened.
  /// - If [`PartialSnapshot::instant`] is [`None`], it means the lex digested 0 bytes.
  ///
  /// You can use [`Lexer::restore`] or [`Lexer::clone_with_snapshot`] to apply this snapshot.
  /// # Caveats
  /// To prevent unnecessary allocations, we only clone the state and the instant if they are mutated
  /// during the lex.
  /// But since these fields might be [`None`], you should use this as soon as possible,
  /// before you further mutate the lexer.
  /// If you want to store this for later use, you should use [`PartialSnapshot::into_full`]
  /// to ensure the snapshot is complete.
  pub snapshot: PartialSnapshot<'text, State>,
  // TODO: add a ref of the lexer as a guard to prevent caller from mutating the lexer
  // before applying the partial snapshot?
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

  #[inline]
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

  #[inline]
  fn build_fork_output(
    stateless: Self::StatelessForkOutputType,
    digested: usize,
    lexer: &Lexer<'text, Kind, State, ErrorType>,
  ) -> Self::ForkOutputType {
    Self::ForkOutputType {
      ctx: stateless.ctx,
      snapshot: PartialSnapshot {
        state: stateless.state,
        instant: (digested != 0).then(|| lexer.instant().clone()),
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::LexerBuilder;

  #[test]
  fn mock_fork_output_factory() {
    let factory = ();
    let stateless_fork_output =
      ForkOutputFactory::<(), i32, ()>::into_stateless_fork_output(factory, 0, 2, 1);
    assert_eq!(stateless_fork_output, ());
    let lexer = LexerBuilder::<()>::new().build("");
    let fork_output =
      <() as ForkOutputFactory<_, _, _>>::build_fork_output(stateless_fork_output, 0, &lexer);
    assert_eq!(fork_output, ());
  }

  #[test]
  fn test_stateless_fork_output() {
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
  fn fork_output_builder() {
    let builder = ForkOutputBuilder::default();
    let stateless_fork_output =
      ForkOutputFactory::<(), i32, ()>::into_stateless_fork_output(builder, 0, 2, 1);
    assert_eq!(
      stateless_fork_output,
      StatelessForkOutput {
        state: Some(0),
        ctx: None
      }
    );
    let lexer = LexerBuilder::<(), _>::stateful().build("");
    let fork_output =
      ForkOutputBuilder::build_fork_output(stateless_fork_output.clone(), 0, &lexer);
    assert_eq!(
      fork_output,
      ForkOutput {
        ctx: None,
        snapshot: PartialSnapshot {
          state: Some(0),
          instant: None
        }
      }
    );
    let fork_output =
      ForkOutputBuilder::build_fork_output(stateless_fork_output.clone(), 1, &lexer);
    assert_eq!(
      fork_output,
      ForkOutput {
        ctx: None,
        snapshot: PartialSnapshot {
          state: Some(0),
          instant: Some(lexer.instant().clone())
        }
      }
    );

    let builder = ForkOutputBuilder::default();
    let stateless_fork_output =
      ForkOutputFactory::<(), i32, ()>::into_stateless_fork_output(builder, 0, 2, 0);
    assert_eq!(
      stateless_fork_output,
      StatelessForkOutput {
        state: Some(0),
        ctx: Some(ReLexContext { start: 0, skip: 1 })
      }
    );
    let mut lexer = LexerBuilder::<(), _>::stateful().build("");
    lexer.state = 1;
    let fork_output =
      ForkOutputBuilder::build_fork_output(stateless_fork_output.clone(), 0, &lexer);
    assert_eq!(
      fork_output,
      ForkOutput {
        ctx: Some(ReLexContext { start: 0, skip: 1 }),
        snapshot: PartialSnapshot {
          state: Some(0),
          instant: None
        }
      }
    );
    let fork_output =
      ForkOutputBuilder::build_fork_output(stateless_fork_output.clone(), 1, &lexer);
    assert_eq!(
      fork_output,
      ForkOutput {
        ctx: Some(ReLexContext { start: 0, skip: 1 }),
        snapshot: PartialSnapshot {
          state: Some(0),
          instant: Some(lexer.instant().clone())
        }
      }
    );
  }
}
