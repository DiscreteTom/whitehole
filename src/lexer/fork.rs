use super::re_lex::{ReLexableBuilder, ReLexableFactory};

/// See [`LexOptions::fork`](crate::lexer::options::LexOptions::fork).
///
/// These types already implement the [`LexOptionsFork`] trait:
/// - `()` - means the fork feature is disabled.
/// - [`ForkEnabled`] - means the fork feature is enabled.
///
/// We use this trait instead of a [`bool`] value
/// to implement the [`fork`](crate::lexer::options::LexOptions::fork) feature
/// so that we can return different types in [`ReLexableFactory::into_re_lexable`]
/// to avoid unnecessary allocations.
pub trait LexOptionsFork<'text, Kind, State, ErrorType> {
  // this has to implement `Default` because the instance is not provided by the user
  // and we have to create the instance by our own
  type ReLexableFactoryType: ReLexableFactory<'text, Kind, State, ErrorType> + Default;
}

// the mock implementation of the fork feature
impl<'text, Kind, State, ErrorType> LexOptionsFork<'text, Kind, State, ErrorType> for () {
  type ReLexableFactoryType = ();
}

/// This struct is used to indicate that the fork feature is enabled.
/// This struct implements [`LexOptionsFork`].
/// See [`LexOptions::fork`](crate::lexer::options::LexOptions::fork).
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct ForkEnabled;

impl<'text, Kind, State: Clone, ErrorType> LexOptionsFork<'text, Kind, State, ErrorType>
  for ForkEnabled
{
  type ReLexableFactoryType = ReLexableBuilder<State>;
}
