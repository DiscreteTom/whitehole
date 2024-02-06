// use crate::{
//   lexer::token::TokenKind,
//   parser::elr::grammar::{grammar::Grammar, grammar_rule::GrammarRule},
// };
// use std::rc::Rc;

// pub struct Candidate<TKind: TokenKind, NTKind: TokenKind> {
//   gr: Rc<GrammarRule<TKind, NTKind>>,
//   digested: usize,
// }

// impl<TKind: TokenKind, NTKind: TokenKind> Candidate<TKind, NTKind> {
//   pub fn current(&self) -> Option<&Rc<Grammar<TKind, NTKind>>> {
//     self.gr.rule().get(self.digested)
//   }
// }
