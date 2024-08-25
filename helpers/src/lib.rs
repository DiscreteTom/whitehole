/// Create an action for each string using
/// [`exact`](https://docs.rs/whitehole/latest/whitehole/lexer/action/utils/exact/fn.exact.html).
///
/// The [`Action::head`](https://docs.rs/whitehole/latest/whitehole/lexer/action/struct.Action.html)
/// will be set automatically.
/// # Panics
/// Panics if any string is empty.
/// # Examples
/// ```
/// use whitehole::lexer::action::{Action, exact_vec, exact};
///
/// # let actions: Vec<Action<_>> =
/// exact_vec!["++", "--"];
/// // equals to
/// vec![exact("++"), exact("--")];
/// ```
#[macro_export]
macro_rules! exact_vec {
  ($($s:expr),*) => {
    vec![$(whitehole::lexer::action::exact($s)),*]
  };
}

/// Same as [`exact_vec`] but only used internally.
#[macro_export]
macro_rules! _exact_vec {
  ($($s:expr),*) => {
    vec![$(crate::lexer::action::exact($s)),*]
  };
}

/// Create an action for each string using
/// [`unchecked_exact`](https://docs.rs/whitehole/latest/whitehole/lexer/action/utils/exact/fn.unchecked_exact.html).
///
/// The [`Action::head`](https://docs.rs/whitehole/latest/whitehole/lexer/action/struct.Action.html)
/// will be set automatically.
/// # Panics
/// Panics if any string is empty.
/// # Caveats
/// You should only use this if you are sure the token is unique by its first char,
/// and the content you are lexing is valid to your format.
/// # Examples
/// ```
/// use whitehole::lexer::action::{Action, unchecked_exact_vec, unchecked_exact};
///
/// # let actions: Vec<Action<_>> =
/// unchecked_exact_vec!["++", "--"];
/// // equals to
/// vec![unchecked_exact("++"), unchecked_exact("--")];
/// ```
#[macro_export]
macro_rules! unchecked_exact_vec {
  ($($s:expr),*) => {
    vec![$(whitehole::lexer::action::unchecked_exact($s)),*]
  };
}

/// Same as [`unchecked_exact_vec`] but only used internally.
#[macro_export]
macro_rules! _unchecked_exact_vec {
  ($($s:expr),*) => {
    vec![$(crate::lexer::action::unchecked_exact($s)),*]
  };
}

/// Create an action for each string using
/// [`word`](https://docs.rs/whitehole/latest/whitehole/lexer/action/utils/word/fn.word.html).
///
/// The [`Action::head`](https://docs.rs/whitehole/latest/whitehole/lexer/action/struct.Action.html)
/// will be set automatically.
/// # Panics
/// Panics if any string is empty.
/// # Examples
/// ```
/// use whitehole::lexer::action::{Action, word_vec, word};
///
/// # let actions: Vec<Action<_>> =
/// word_vec!["int", "bool"];
/// // equals to
/// vec![word("int"), word("bool")];
/// ```
#[macro_export]
macro_rules! word_vec {
  ($($s:expr),*) => {
    vec![$(whitehole::lexer::action::word($s)),*]
  };
}

/// Same as [`word_vec`] but only used internally.
#[macro_export]
macro_rules! _word_vec {
  ($($s:expr),*) => {
    vec![$(crate::lexer::action::word($s)),*]
  };
}
