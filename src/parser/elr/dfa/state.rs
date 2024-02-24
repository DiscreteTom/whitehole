use super::{
  candidate::{Candidate, CandidateId},
  utils::lex_grammar,
};
use crate::{
  lexer::{
    expectation::Expectation,
    token::{Token, TokenKind, TokenKindId},
    trimmed::TrimmedLexer,
  },
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
    TKind: TokenKind<TKind> + 'static,
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

  pub fn id(&self) -> &StateId {
    &self.id
  }
  pub fn candidates(
    &self,
  ) -> &Vec<
    Rc<Candidate<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
  > {
    &self.candidates
  }
  pub fn next_map(&self) -> &HashMap<GrammarId, Option<StateId>> {
    &self.next_map
  }
  pub fn conflict_map(
    &self,
  ) -> &HashMap<CandidateId, Vec<Conflict<GrammarRuleId, Rc<Grammar<TKind, NTKind>>>>> {
    &self.conflict_map
  }
}

pub struct StatefulState<
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
> {
  core: Rc<State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
  next_expectational_lex_index: usize,
  lexed_grammars: HashSet<GrammarId>,
  lexed_without_expectation: bool,
}

impl<
    TKind: TokenKind<TKind> + 'static,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Default + Clone + 'static,
    LexerErrorType: 'static,
  > From<Rc<State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>>
  for StatefulState<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>
{
  fn from(
    core: Rc<State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
  ) -> Self {
    Self {
      core,
      next_expectational_lex_index: 0,
      lexed_grammars: HashSet::new(),
      lexed_without_expectation: false,
    }
  }
}

impl<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Default + Clone + 'static,
    LexerErrorType: 'static,
  > StatefulState<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>
{
  pub fn try_lex<'buffer>(
    &mut self,
    lexer: &TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    // TODO: add param token_ast_mapper
    global: &Rc<RefCell<Global>>,
  ) -> Option<
    StateTryLexOutput<
      ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>,
      TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    >,
  > {
    // first, try expectational lex
    for (i, c) in self.core.candidates()[self.next_expectational_lex_index..]
      .iter()
      .enumerate()
    {
      if let Some(output) = c.try_lex_with_expectation(lexer, &mut self.lexed_grammars, global) {
        self.next_expectational_lex_index = i + 1;
        return Some(StateTryLexOutput {
          node: output.node,
          lexer: output.lexer,
          next_state_id: self.get_next_by_lexed_grammar(&output.grammar_id).clone(),
        });
      }
    }

    // no candidate can lex with expectation, try to lex without expectation
    if self.lexed_without_expectation {
      // already lexed without expectation, no need to try again
      return None;
    }
    // mark lexed without expectation as done no matter if the lex is successful
    self.lexed_without_expectation = true;

    // lex without expectation
    self.next_expectational_lex_index = self.core.candidates().len(); // no next expectational lex
    lex_grammar(Expectation::default(), lexer, global).and_then(|output| {
      self
        .try_accept_t_node_without_expectation(&output.t_kind_id, output.text)
        .map(|next_state_id| {
          StateTryLexOutput {
            node: output.node,
            lexer: output.lexer,
            // treat as the candidate lexed the node
            next_state_id: next_state_id.clone(),
          }
        })
    })
  }

  pub fn try_reduce<'buffer>(
    &self,
    buffer: &Vec<ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>>,
    next_token: &Option<Token<'buffer, TKind, LexerErrorType>>,
    lexer: &TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    reducing_stack: &Vec<usize>,
    entry_nts: &HashSet<GrammarId>,
    follow_sets: &HashMap<GrammarId, HashSet<Rc<Grammar<TKind, NTKind>>>>,
  ) -> Option<StateTryReduceOutput<ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>>> {
    for c in self.core.candidates().iter() {
      if let Some(output) = c.try_reduce(
        buffer,
        next_token,
        lexer,
        reducing_stack,
        entry_nts,
        follow_sets,
        self.core.conflict_map().get(c.id()),
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

  pub fn try_accept_t_node_without_expectation<'buffer>(
    &self,
    t_kind_id: &TokenKindId<TKind>,
    text: &'buffer str,
  ) -> Option<&StateId> {
    // check if any candidate can accept the lexed node
    for c in self.core.candidates() {
      if let Some(grammar_id) = c.try_accept_t_node_without_expectation(t_kind_id, text) {
        return Some(self.get_next_by_lexed_grammar(&grammar_id));
      }
    }
    // no candidate can accept the lexed node
    None
  }

  fn get_next_by_lexed_grammar(&self, grammar_id: &GrammarId) -> &StateId {
    match self.core.next_map().get(grammar_id) {
      // cache miss. this should never happen since when building DFA
      // we should already calculated the next state for all grammars in rules
      // see [[@get_all_grammar_id_from_rules]]
      None => unreachable!(
        "{:?} next cache miss by lexed {:?}",
        self.core.id(),
        grammar_id
      ),
      // cache hit
      Some(hit) => match hit {
        // cache hit but no next state
        // this should never happen since if a grammar can be lexed by a candidate
        // the candidate must have a next candidate and thus
        // this state must have a next state
        None => unreachable!(
          "Lexed {:?} is not acceptable by {:?}",
          grammar_id,
          self.core.id()
        ),
        Some(next) => next,
      },
    }
  }

  // [[get_next_by_reduced_grammar]]
  pub fn get_next_by_reduced_grammar(&self, grammar_id: &GrammarId) -> Option<StateId> {
    self
      .core
      .next_map()
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
          grammar_id,
          self.core.id()
        ),
        Some(next) => Some(next.clone()),
      })
  }

  /// Reset the stateful state so that all cache is cleared.
  /// This should be called when the lexer's state is reset.
  pub fn reset(&mut self) {
    self.lexed_grammars.clear();
    self.lexed_without_expectation = false;
    self.next_expectational_lex_index = 0;
  }
}

pub struct StateTryLexOutput<NodeType, LexerType> {
  pub node: NodeType,
  pub lexer: LexerType,
  pub next_state_id: StateId,
}

pub struct StateTryReduceOutput<NodeType> {
  pub node: NodeType,
  pub nt_grammar_id: GrammarId,
  pub reduced: usize,
}
