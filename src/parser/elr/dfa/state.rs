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

pub struct State<Kind: TokenKind + Clone, ASTData: 'static, ErrorType: 'static, Global: 'static> {
  id: StateId,
  candidates: Rc<Vec<Rc<GrammarRule<Kind, ASTData, ErrorType, Global>>>>,
  max_candidate_length: usize,
  digested: usize,
  next_map: HashMap<GrammarId, Option<Rc<Self>>>,
}

impl<Kind: TokenKind + Clone, ASTData: 'static, ErrorType: 'static, Global: 'static>
  State<Kind, ASTData, ErrorType, Global>
{
  pub fn new(
    id: StateId,
    candidates: Rc<Vec<Rc<GrammarRule<Kind, ASTData, ErrorType, Global>>>>,
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

  pub fn generate_next(&self, input: &Grammar<Kind>) {}

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
        let next = match { self.next_map.get(&output.grammar_id) } {
          // this should never be None, since when building DFA
          // we should already calculated the next state in generate_next for all grammars
          // TODO: don't panic, return Err?
          None => panic!("No next state for grammar {:?}", output.grammar_id),
          Some(next) => match next {
            // here the next state is None (no candidates), should try next grammar rule
            // TODO: is this never happen?
            None => continue,
            Some(next) => next.clone(),
          },
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

pub struct StateTryLexOutput<NodeType, LexerType, StateType> {
  pub node: NodeType,
  pub lexer: LexerType,
  pub next_candidate_index: usize,
  pub next_state: StateType,
}

pub struct StateTryReduceOutput<NodeType> {
  pub node: NodeType,
  pub reduced: usize,
}
