use super::re_lex::{ReLexableBuilder, ReLexableFactory};

/// See [`LexOptions::fork`](crate::lexer::options::LexOptions::fork).
// we use this trait and 2 structs instead of a `bool` to implement the `Fork` feature
// so that we can return different types in `into_re_lexable` to avoid unnecessary allocations
pub trait LexOptionsFork<'text, Kind: 'static, ActionState, ErrorType, ErrAcc> {
  type ReLexableFactoryType: ReLexableFactory<'text, Kind, ActionState, ErrorType, ErrAcc> + Default;
}

/// This struct is used to indicate that the fork feature is enabled.
/// This struct implements [`LexOptionsFork`].
/// See [`LexOptions::fork`](crate::lexer::options::LexOptions::fork).
pub struct ForkEnabled;

impl<'text, Kind: 'static, ActionState: Clone, ErrorType, ErrAcc: Clone>
  LexOptionsFork<'text, Kind, ActionState, ErrorType, ErrAcc> for ForkEnabled
{
  type ReLexableFactoryType = ReLexableBuilder<ActionState>;
}

/// This struct is used to indicate that the fork feature is disabled.
/// This struct implements [`LexOptionsFork`].
/// See [`LexOptions::fork`](crate::lexer::options::LexOptions::fork).
pub struct ForkDisabled;

impl<'text, Kind: 'static, ActionState, ErrorType, ErrAcc>
  LexOptionsFork<'text, Kind, ActionState, ErrorType, ErrAcc> for ForkDisabled
{
  type ReLexableFactoryType = ();
}
