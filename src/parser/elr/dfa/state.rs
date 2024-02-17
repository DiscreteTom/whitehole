use super::{
  candidate::{Candidate, CandidateId},
  utils::lex_grammar,
};
use crate::{
  lexer::{expectation::Expectation, token::TokenKind, trimmed::TrimmedLexer},
  parser::{
    ast::ASTNode,
    elr::{
      builder::conflict::Conflict,
      grammar::{
        grammar::{Grammar, GrammarId},
        grammar_rule::GrammarRuleId,
      },
    },
  },
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
  conflict_map: HashMap<CandidateId, Vec<Conflict<GrammarRuleId, Rc<Grammar<TKind, NTKind>>>>>,
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
    conflict_map: HashMap<CandidateId, Vec<Conflict<GrammarRuleId, Rc<Grammar<TKind, NTKind>>>>>,
  ) -> Self {
    Self {
      id,
      candidates,
      next_map,
      conflict_map,
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
    // TODO: store lexed_grammars and lexed_without_expectation in State
    lexed_grammars: &mut HashSet<GrammarId>,
    lexed_without_expectation: &mut bool,
    global: &Rc<RefCell<Global>>,
  ) -> Option<
    StateTryLexOutput<
      ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>,
      TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    >,
  > {
    // first, try expectational lex
    for (i, c) in self.candidates[from_index..].iter().enumerate() {
      if let Some(output) = c.try_lex_with_expectation(lexer, lexed_grammars, global) {
        return Some(StateTryLexOutput {
          node: output.node,
          lexer: output.lexer,
          next_expectational_lex_candidate_index: i + 1,
          next_state_id: self.get_next_by_lexed_grammar(&output.grammar_id).clone(),
        });
      }
    }

    // no candidate can lex with expectation, try to lex without expectation
    if *lexed_without_expectation {
      // already lexed without expectation, no need to try again
      return None;
    }
    // mark lexed without expectation as done no matter if the lex is successful
    *lexed_without_expectation = true;

    // lex without expectation
    lex_grammar(Expectation::default(), lexer, global).and_then(|output| {
      // check if any candidate can accept the lexed node
      for c in &self.candidates {
        if let Some(grammar_id) =
          c.try_accept_t_node_without_expectation(&output.t_kind_id, output.text)
        {
          return Some(StateTryLexOutput {
            node: output.node,
            lexer: output.lexer,
            next_expectational_lex_candidate_index: self.candidates.len(), // no next expectational lex
            // treat as the candidate lexed the node
            next_state_id: self.get_next_by_lexed_grammar(&grammar_id).clone(),
          });
        }
      }

      // no candidate can accept the lexed node
      None
    })
  }

  pub fn try_reduce<'buffer>(
    &self,
    buffer: &Vec<ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>>,
    lexer: &TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    reducing_stack: &Vec<usize>,
    entry_nts: &HashSet<GrammarId>,
    follow_sets: &HashMap<GrammarId, HashSet<Rc<Grammar<TKind, NTKind>>>>,
  ) -> Option<StateTryReduceOutput<ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>>> {
    for c in self.candidates.iter() {
      if let Some(output) = c.try_reduce(
        buffer,
        lexer,
        reducing_stack,
        entry_nts,
        follow_sets,
        self.conflict_map.get(c.id()).unwrap(),
      ) {
        return Some(StateTryReduceOutput {
          node: output.node,
          nt_grammar_id: output.nt_grammar_id,
          reduced: output.reduced,
        });
      }
    }
    None
  }

  fn get_next_by_lexed_grammar(&self, grammar_id: &GrammarId) -> &StateId {
    match self.next_map.get(grammar_id) {
      // cache miss. this should never happen since when building DFA
      // we should already calculated the next state for all grammars in rules
      // see [[@get_all_grammar_id_from_rules]]
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

  // [[get_next_by_reduced_grammar]]
  pub fn get_next_by_reduced_grammar(&self, grammar_id: &GrammarId) -> Option<StateId> {
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
  pub next_expectational_lex_candidate_index: usize,
  pub next_state_id: StateId,
}

pub struct StateTryReduceOutput<NodeType> {
  pub node: NodeType,
  pub nt_grammar_id: GrammarId,
  pub reduced: usize,
}
