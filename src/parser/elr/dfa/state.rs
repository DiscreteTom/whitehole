use crate::{
  lexer::{
    token::{TokenKind, TokenKindId},
    trimmed::TrimmedLexer,
  },
  parser::{
    ast::ASTNode,
    elr::grammar::{grammar::GrammarId, grammar_rule::GrammarRule},
  },
};
use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
  rc::Rc,
};

pub struct State<Kind: TokenKind + Clone, ASTData: 'static, ErrorType: 'static, Global: 'static> {
  candidates: Rc<Vec<Rc<GrammarRule<Kind, ASTData, ErrorType, Global>>>>,
  max_candidate_length: usize,
  digested: usize,
}

impl<Kind: TokenKind + Clone, ASTData: 'static, ErrorType: 'static, Global: 'static> Clone
  for State<Kind, ASTData, ErrorType, Global>
{
  fn clone(&self) -> Self {
    Self {
      candidates: self.candidates.clone(),
      max_candidate_length: self.max_candidate_length,
      digested: self.digested,
    }
  }
}

impl<Kind: TokenKind + Clone, ASTData: 'static, ErrorType: 'static, Global: 'static>
  State<Kind, ASTData, ErrorType, Global>
{
  pub fn get_next(&self) -> Option<Self> {
    if self.digested < self.max_candidate_length - 1 {
      Some(Self {
        candidates: self.candidates.clone(),
        max_candidate_length: self.max_candidate_length,
        digested: self.digested + 1,
      })
    } else {
      None
    }
  }

  pub fn try_lex<'buffer, LexerActionState: Default + Clone, LexerErrorType>(
    &self,
    lexer: &TrimmedLexer<'buffer, Kind, LexerActionState, LexerErrorType>,
    // TODO: add param token_ast_mapper
    from_index: usize,
    lexed_grammars: &mut HashSet<GrammarId>,
    lexed_without_expectation: &mut bool,
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

  pub fn try_reduce<'buffer, LexerActionState: Default + Clone, LexerErrorType>(
    &self,
    buffer: &Vec<ASTNode<Kind, ASTData, ErrorType, Global>>,
    lexer: &TrimmedLexer<'buffer, Kind, LexerActionState, LexerErrorType>,
    reducing_stack: &Vec<usize>,
    entry_nts: &HashSet<TokenKindId>,
    follow_sets: &HashMap<TokenKindId, TokenKindId>,
  ) -> Option<StateTryReduceOutput<ASTNode<Kind, ASTData, ErrorType, Global>>> {
    for c in self.candidates.iter() {
      if let Some(node) = c.try_reduce(
        self.digested,
        buffer,
        lexer,
        reducing_stack,
        entry_nts,
        follow_sets,
      ) {
        return Some(StateTryReduceOutput {
          node,
          reduced: c.rule().len(),
        });
      }
    }
    None
  }
}

pub struct StateTryLexOutput<NodeType, LexerType> {
  pub node: NodeType,
  pub lexer: LexerType,
  pub next_candidate_index: usize,
}

pub struct StateTryReduceOutput<NodeType> {
  pub node: NodeType,
  pub reduced: usize,
}
