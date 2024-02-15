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
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
> {
  id: StateId,
  candidates:
    Vec<Rc<Candidate<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>>,
  next_map: HashMap<GrammarId, Option<StateId>>,
}

impl<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Default + Clone + 'static,
    LexerErrorType: 'static,
  > State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>
{
  pub fn new(
    id: StateId,
    candidates: Vec<
      Rc<Candidate<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
    >,
    next_map: HashMap<GrammarId, Option<StateId>>,
  ) -> Self {
    Self {
      id,
      candidates,
      next_map,
    }
  }

  pub fn candidates(
    &self,
  ) -> &Vec<
    Rc<Candidate<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
  > {
    &self.candidates
  }

  pub fn try_lex<'buffer>(
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
        return Some(StateTryLexOutput {
          node: output.node,
          lexer: output.lexer,
          next_candidate_index: i + 1,
          next_state_id: self.get_next_by_lexed_grammar(&output.grammar_id).clone(),
        });
      }
    }
    // no candidate matches
    None
  }

  pub fn try_reduce<'buffer>(
    &self,
    buffer: &Vec<ASTNode<TKind, NTKind, ASTData, ErrorType, Global>>,
    lexer: &TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    reducing_stack: &Vec<usize>,
    entry_nts: &HashSet<TokenKindId<NTKind>>,
    follow_sets: &HashMap<GrammarId, HashSet<GrammarId>>,
  ) -> Option<StateTryReduceOutput<ASTNode<TKind, NTKind, ASTData, ErrorType, Global>>> {
    for c in self.candidates.iter() {
      if let Some(output) = c.try_reduce(buffer, lexer, reducing_stack, entry_nts, follow_sets) {
        return Some(StateTryReduceOutput {
          node: output.node,
          nt_grammar_id: output.nt_grammar_id,
          reduced: output.reduced,
          next_state_id: self.get_next_by_reduced_grammar(&output.nt_grammar_id),
        });
      }
    }
    None
  }

  fn get_next_by_lexed_grammar(&self, grammar_id: &GrammarId) -> &StateId {
    match self.next_map.get(grammar_id) {
      // cache miss. this should never happen since when building DFA
      // we should already calculated the next state for all grammars in rules
      None => unreachable!("{:?} next cache miss by lexed {:?}", self.id, grammar_id),
      // cache hit
      Some(hit) => match hit {
        // cache hit but no next state
        // this should never happen since if a grammar can be lexed by a candidate
        // the candidate must have a next candidate and thus
        // this state must have a next state
        None => unreachable!("Lexed {:?} is not acceptable by {:?}", grammar_id, self.id),
        Some(next) => next,
      },
    }
  }

  fn get_next_by_reduced_grammar(&self, grammar_id: &GrammarId) -> Option<StateId> {
    self
      .next_map
      .get(grammar_id)
      // cache miss is acceptable here
      // because when the reduced grammar is an entry-only NT
      // the cache should miss and the parsing process should be done
      .and_then(|hit| match hit {
        // cache hit but no next state
        // this should never happen since when we construct the state
        // with NT closures, the reduced candidate should belong
        // to another candidate's NT closure.
        None => unreachable!(
          "Reduced {:?} is not acceptable by {:?}",
          grammar_id, self.id
        ),
        Some(next) => Some(next.clone()),
      })
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
  pub next_state_id: Option<StateId>,
}
