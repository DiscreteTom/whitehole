use super::re_lex::ReLexContext;

/// See [`LexOptions::fork`](crate::lexer::options::LexOptions::fork).
///
/// These types already implement the [`LexOptionsFork`] trait:
/// - `()` - means the fork feature is disabled.
/// - [`ForkEnabled`] - means the fork feature is enabled.
///
/// We use this trait instead of a [`bool`] value
/// to implement the [`fork`](crate::lexer::options::LexOptions::fork) feature
/// so that we can return different types in [`ForkOutputFactory::into_fork_output`]
/// to avoid unnecessary allocations.
pub trait LexOptionsFork {
  // this has to implement `Default` because the instance is not provided by the user
  // and we have to create the instance by our own
  type OutputFactoryType: ForkOutputFactory + Default;
}

// the mock implementation of the fork feature
impl LexOptionsFork for () {
  type OutputFactoryType = ();
}

/// This struct is used to indicate that the fork feature is enabled.
/// This struct implements [`LexOptionsFork`].
/// See [`LexOptions::fork`](crate::lexer::options::LexOptions::fork).
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct ForkEnabled;

impl LexOptionsFork for ForkEnabled {
  type OutputFactoryType = ForkOutputBuilder;
}

/// These types already implement the [`ForkOutputFactory`] trait:
/// - `()` - no fork output will be created.
/// - [`ForkOutputBuilder`] - create fork output structs if possible.
pub trait ForkOutputFactory {
  /// This should extends [`Default`] so when no token is emitted,
  /// the output can be created with a default value.
  type ForkOutputType: Default;

  fn into_fork_output(
    self,
    start: usize,
    actions_len: usize,
    action_index: usize,
  ) -> Self::ForkOutputType;
}

// mock fork output factory
impl ForkOutputFactory for () {
  type ForkOutputType = ();

  #[inline]
  fn into_fork_output(
    self,
    _start: usize,
    _actions_len: usize,
    _action_index: usize,
  ) -> Self::ForkOutputType {
  }
}

#[derive(Default, Debug)]
pub struct ForkOutputBuilder;

impl ForkOutputFactory for ForkOutputBuilder {
  type ForkOutputType = Option<ReLexContext>;

  #[inline]
  fn into_fork_output(
    self,
    start: usize,
    actions_len: usize,
    action_index: usize,
  ) -> Self::ForkOutputType {
    if action_index < actions_len - 1 {
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
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mock_fork_output_factory() {
    let factory = ();
    let stateless_fork_output =
      ForkOutputFactory::into_fork_output(factory, 0, 2, 1);
    assert_eq!(stateless_fork_output, ());
  }

  #[test]
  fn fork_output_builder() {
    let builder = ForkOutputBuilder::default();
    let stateless_fork_output =
      ForkOutputFactory::into_fork_output(builder, 0, 2, 1);
    assert_eq!(stateless_fork_output, None);

    let builder = ForkOutputBuilder::default();
    let stateless_fork_output =
      ForkOutputFactory::into_fork_output(builder, 0, 2, 0);
    assert_eq!(
      stateless_fork_output,
      Some(ReLexContext { start: 0, skip: 1 })
    );
  }
}
