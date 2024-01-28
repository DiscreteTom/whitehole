use crate::{lexer::token::TokenKind, parser::elr::grammar::grammar_rule::GrammarRule};

pub struct Candidate<'grammar, 'gr, TKind: TokenKind, NTKind> {
  gr: &'gr GrammarRule<'grammar, TKind, NTKind>,
  digested: usize,
}
