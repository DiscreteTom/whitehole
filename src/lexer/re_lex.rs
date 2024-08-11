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
/// let (mut lexer, context) = output.fork.into_lexer(&lexer).unwrap();
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn re_lex_context() {
    let context = ReLexContext::new();
    assert_eq!(context, ReLexContext { start: 0, skip: 0 });
    let context = ReLexContext::default();
    assert_eq!(context, ReLexContext { start: 0, skip: 0 });
  }
}
