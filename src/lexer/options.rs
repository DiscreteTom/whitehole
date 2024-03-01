use super::{expectation::Expectation, output::ReLexContext};

pub struct LexOptions<'expect_text, Kind: 'static> {
  pub expectation: Expectation<'expect_text, Kind>,
  /// If `true`, the [`LexOutput::re_lex`] might be `Some`.
  pub fork: bool,
  /// Provide this if the lex is a re-lex.
  pub re_lex: Option<ReLexContext>,
}
