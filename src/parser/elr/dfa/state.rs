use crate::{
  lexer::{token::TokenKind, trimmed::TrimmedLexer},
  parser::{ast::ASTNode, elr::grammar::grammar_rule::GrammarRule},
};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

pub struct State<Kind: TokenKind> {
  candidates: Vec<Rc<GrammarRule<Kind>>>,
  digested: usize,
}

impl<Kind: TokenKind> State<Kind> {
  pub fn try_lex<
    'buffer,
    ASTData,
    ErrorType,
    LexerActionState: Default + Clone,
    LexerErrorType,
    Global,
  >(
    &self,
    lexer: &TrimmedLexer<'buffer, Kind, LexerActionState, LexerErrorType>,
    // TODO: add param token_ast_mapper
    from_index: usize,
    lexed_grammars: &mut HashSet<usize>,
    lexed_without_expectation: bool,
    global: &Rc<RefCell<Global>>,
  ) -> Option<
    StateTryLexOutput<
      ASTNode<Kind, ASTData, ErrorType, Global>,
      TrimmedLexer<'buffer, Kind, LexerActionState, LexerErrorType>,
    >,
  > {
    for (i, gr) in self.candidates[from_index..].iter().enumerate() {
      if let Some(output) = gr.try_lex(
        self.digested,
        lexer,
        lexed_grammars,
        lexed_without_expectation,
        global,
      ) {
        return Some(StateTryLexOutput {
          node: output.node,
          lexer: output.lexer,
          next_candidate_index: i + 1,
        });
      }
    }
    // no candidate matches
    None
  }
}

pub struct StateTryLexOutput<NodeType, LexerType> {
  pub node: NodeType,
  pub lexer: LexerType,
  pub next_candidate_index: usize,
}
