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
  collections::{HashMap, HashSet},
  rc::Rc,
};

pub struct DfaParseOutput<Kind: TokenKind, ASTData: 'static, ErrorType: 'static, Global: 'static> {
  pub buffer: Vec<ASTNode<Kind, ASTData, ErrorType, Global>>,
}

pub struct Dfa<Kind: TokenKind> {
  grs: GrammarRuleRepo<Kind>,
  entry_nts: HashSet<TokenKindId>,
  entry_state: Rc<State<Kind>>,
  follow_sets: HashMap<TokenKindId, TokenKindId>,
  grammars: GrammarRepo<Kind>,
  // TODO: token_ast_mapper
}

impl<Kind: TokenKind> Dfa<Kind> {
  // pub fn parse<
  //   'buffer,
  //   ASTData: 'static,
  //   ErrorType: 'static,
  //   Global: 'static,
  //   LexerActionState: Default + Clone,
  //   LexerErrorType,
  // >(
  //   &self,
  //   lexer: TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
  // ) -> DfaParseOutput<Kind, ASTData, ErrorType, Global> {
  //   let mut parsing_state = ParsingState {
  //     buffer: Vec::new(),
  //     index: 0,
  //     state_stack: Stack::new(vec![self.entry_state.clone()]),
  //     lexer,
  //   };

  //   loop {
  //     // if no enough AST nodes
  //     if (parsing_state.index >= parsing_state.buffer.len()) {
  //       // try to lex a new one
  //       parsing_state.state_stack.current().unwrap()
  //     }
  //   }
  // }
}
