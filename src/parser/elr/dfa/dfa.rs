use super::{
  parsing::{ParsingState, Stack},
  state::State,
};
use crate::{
  lexer::{
    token::{TokenKind, TokenKindId},
    trimmed::TrimmedLexer,
  },
  parser::{
    ast::ASTNode,
    elr::grammar::{grammar_repo::GrammarRepo, grammar_rule_repo::GrammarRuleRepo},
  },
};
use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
  rc::Rc,
};

pub struct DfaParseOutput<
  Kind: TokenKind + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  pub buffer: Vec<ASTNode<Kind, ASTData, ErrorType, Global>>,
}

pub struct Dfa<Kind: TokenKind + Clone, ASTData: 'static, ErrorType: 'static, Global: 'static> {
  grs: GrammarRuleRepo<Kind, ASTData, ErrorType, Global>,
  entry_nts: HashSet<TokenKindId>,
  entry_state: State<Kind, ASTData, ErrorType, Global>,
  follow_sets: HashMap<TokenKindId, TokenKindId>,
  grammars: GrammarRepo<Kind>,
  // TODO: token_ast_mapper
}

impl<Kind: TokenKind + Clone, ASTData: 'static, ErrorType: 'static, Global: 'static>
  Dfa<Kind, ASTData, ErrorType, Global>
{
  pub fn parse<'buffer, LexerActionState: Default + Clone, LexerErrorType>(
    &self,
    buffer: Vec<ASTNode<Kind, ASTData, ErrorType, Global>>,
    lexer: TrimmedLexer<'buffer, Kind, LexerActionState, LexerErrorType>,
    global: &Rc<RefCell<Global>>,
    // ) -> DfaParseOutput<Kind, ASTData, ErrorType, Global> {
  ) -> () {
    let mut parsing_state = ParsingState {
      buffer,
      state_stack: Stack::new(vec![self.entry_state.clone()]),
      reducing_stack: Vec::new(), // empty since no ASTNode in buffer
      lexer,
      need_lex: true,   // at the beginning we should lex for a new AST node
      try_lex_index: 0, // from the first candidate
      lexed_grammars: HashSet::new(), // no grammars are lexed at the beginning
      lexed_without_expectation: false, // non-expectational lex is not done at the beginning
      errors: Vec::new(),
    };

    loop {
      if parsing_state.need_lex {
        // try to lex a new one
        if parsing_state.try_lex(global) {
          continue;
        } else {
          // TODO: enter panic mode
          todo!()
        }
      }

      // else, no need to lex, just try to reduce
      if !parsing_state.try_reduce(&self.entry_nts, &self.follow_sets) {
        // reduce failed, try to lex more
        continue;
      }

      // else, reduce success
      if self
        .entry_nts
        .contains(&parsing_state.buffer.last().unwrap().kind.id())
        && parsing_state.reducing_stack.len() == 1
      {
        // if the last ASTNode is an entry NT, then parsing is done
        return;
      }

      // else, should try reduce again, just continue
    }
  }
}
