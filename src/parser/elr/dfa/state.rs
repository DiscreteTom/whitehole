use super::candidate::Candidate;
use crate::{
  lexer::{
    token::{TokenKind, TokenKindId},
    trimmed::TrimmedLexer,
  },
  parser::{ast::ASTNode, elr::grammar::grammar::GrammarId},
};
use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
  rc::Rc,
};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, PartialOrd, Ord)]
pub struct StateId(pub usize);

pub struct State<
  TKind: TokenKind<TKind>,
  NTKind: TokenKind<NTKind> + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  id: StateId,
  candidates: Vec<Rc<Candidate<TKind, NTKind, ASTData, ErrorType, Global>>>,
  next_map: HashMap<GrammarId, Option<StateId>>,
}

impl<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  > State<TKind, NTKind, ASTData, ErrorType, Global>
{
  pub fn new(
    id: StateId,
    candidates: Vec<Rc<Candidate<TKind, NTKind, ASTData, ErrorType, Global>>>,
    next_map: HashMap<GrammarId, Option<StateId>>,
  ) -> Self {
    Self {
      id,
      candidates,
      next_map,
    }
  }

  pub fn candidates(&self) -> &Vec<Rc<Candidate<TKind, NTKind, ASTData, ErrorType, Global>>> {
    &self.candidates
  }

  pub fn try_lex<'buffer, LexerActionState: Default + Clone, LexerErrorType>(
    &self,
    lexer: &TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    // TODO: add param token_ast_mapper
    from_index: usize,
    lexed_grammars: &mut HashSet<GrammarId>,
    lexed_without_expectation: &mut bool,
    global: &Rc<RefCell<Global>>,
  ) -> Option<
    StateTryLexOutput<
      ASTNode<TKind, NTKind, ASTData, ErrorType, Global>,
      TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    >,
  > {
    for (i, c) in self.candidates[from_index..].iter().enumerate() {
      if let Some(output) = c.try_lex(lexer, lexed_grammars, lexed_without_expectation, global) {
        // get the next state by the lexed grammar
        let next = match self.get_next(&output.grammar_id) {
          // no next state, continue to try next candidate
          // TODO: will this happen?
          None => continue,
          Some(next) => next,
        };

        return Some(StateTryLexOutput {
          node: output.node,
          lexer: output.lexer,
          next_candidate_index: i + 1,
          next_state_id: next,
        });
      }
    }
    // no candidate matches
    None
  }

  pub fn try_reduce<'buffer, LexerActionState: Default + Clone, LexerErrorType>(
    &self,
    buffer: &Vec<ASTNode<TKind, NTKind, ASTData, ErrorType, Global>>,
    lexer: &TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    reducing_stack: &Vec<usize>,
    entry_nts: &HashSet<TokenKindId<NTKind>>,
    follow_sets: &HashMap<GrammarId, HashSet<GrammarId>>,
  ) -> Option<StateTryReduceOutput<ASTNode<TKind, NTKind, ASTData, ErrorType, Global>>> {
    for c in self.candidates.iter() {
      if let Some(output) = c.try_reduce(buffer, lexer, reducing_stack, entry_nts, follow_sets) {
        // TODO: just return candidate output
        return Some(StateTryReduceOutput {
          node: output.node,
          nt_grammar_id: output.nt_grammar_id,
          reduced: output.reduced,
        });
      }
    }
    None
  }

  pub fn get_next(&self, grammar_id: &GrammarId) -> Option<StateId> {
    match self.next_map.get(grammar_id) {
      // when building DFA
      // we should already calculated the next state for all grammars
      None => unreachable!("No next state for grammar {:?}", grammar_id),
      // here the next state still may be None (no candidates)
      // usually happen when try_reduce
      // TODO: is the comment correct?
      Some(next) => next.clone(),
    }
  }
}

pub struct StateTryLexOutput<NodeType, LexerType> {
  pub node: NodeType,
  pub lexer: LexerType,
  pub next_candidate_index: usize,
  pub next_state_id: StateId,
}

pub struct StateTryReduceOutput<NodeType> {
  pub node: NodeType,
  pub nt_grammar_id: GrammarId,
  pub reduced: usize,
}
