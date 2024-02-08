use crate::{
  lexer::{
    token::{TokenKind, TokenKindId},
    trimmed::TrimmedLexer,
  },
  parser::{
    ast::ASTNode,
    elr::grammar::{
      grammar::{Grammar, GrammarId},
      grammar_rule::GrammarRule,
    },
  },
};
use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
  rc::Rc,
};

pub type StateId = usize;

pub struct State<
  TKind: TokenKind,
  NTKind: TokenKind,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  id: StateId,
  candidates: Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>>,
  max_candidate_length: usize,
  digested: usize,
  next_map: HashMap<GrammarId, Option<Rc<Self>>>,
}

impl<
    TKind: TokenKind,
    NTKind: TokenKind + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  > State<TKind, NTKind, ASTData, ErrorType, Global>
{
  pub fn new(
    id: StateId,
    candidates: Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>>,
    digested: usize,
  ) -> Self {
    Self {
      id,
      max_candidate_length: candidates.iter().map(|c| c.rule().len()).max().unwrap(),
      candidates,
      digested,
      next_map: HashMap::new(),
    }
  }

  pub fn candidates(&self) -> &[Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>] {
    &self.candidates
  }

  // TODO: only available when enable feature `generate`?
  pub fn generate_next(&self, input: &Grammar<TKind, NTKind>) {}

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
      Rc<Self>,
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
          next_state: next,
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
    entry_nts: &HashSet<TokenKindId>,
    follow_sets: &HashMap<TokenKindId, TokenKindId>,
  ) -> Option<StateTryReduceOutput<ASTNode<TKind, NTKind, ASTData, ErrorType, Global>, Rc<Self>>>
  {
    for c in self.candidates.iter() {
      if let Some(node) = c.try_reduce(
        self.digested,
        buffer,
        lexer,
        reducing_stack,
        entry_nts,
        follow_sets,
      ) {
        // get the next state by the reduced grammar (NT)
        let next = match self.get_next(&c.nt().id()) {
          // no next state, continue to try next candidate
          // TODO: will this happen?
          None => continue,
          Some(next) => next,
        };
        return Some(StateTryReduceOutput {
          node,
          reduced: c.rule().len(),
          next_state: next,
        });
      }
    }
    None
  }

  fn get_next(&self, grammar_id: &GrammarId) -> Option<Rc<Self>> {
    match self.next_map.get(grammar_id) {
      // this should never be None, since when building DFA
      // we should already calculated the next state in generate_next for all grammars
      // TODO: don't panic, return Err?
      None => panic!("No next state for grammar {:?}", grammar_id),
      // here the next state still may be None (no candidates)
      // usually happen when try_reduce
      // TODO: is the comment correct?
      Some(next) => next.clone(),
    }
  }
}

pub struct StateTryLexOutput<NodeType, LexerType, StateType> {
  pub node: NodeType,
  pub lexer: LexerType,
  pub next_candidate_index: usize,
  pub next_state: StateType,
}

pub struct StateTryReduceOutput<NodeType, StateType> {
  pub node: NodeType,
  pub reduced: usize,
  pub next_state: StateType,
}
