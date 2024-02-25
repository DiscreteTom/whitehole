use super::{
  parsing::{ParsingState, TryReduceResult},
  stack::Stack,
  state::{State, StateId},
};
use crate::{
  lexer::{token::TokenKind, trimmed::TrimmedLexer},
  parser::{
    ast::ASTNode,
    elr::{
      builder::lexer_panic_handler::LexerPanicHandler,
      grammar::grammar::{Grammar, GrammarId},
    },
  },
};
use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
  rc::Rc,
};

pub struct DfaParseOutput<
  'buffer,
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
> {
  pub lexer: TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
  pub buffer: Vec<ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>>,
  pub errors: Vec<usize>,
}

pub struct Dfa<
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
> {
  entry_nts: HashSet<GrammarId>,
  entry_state:
    Rc<State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
  states: HashMap<
    StateId,
    Rc<State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
  >,
  follow_sets: HashMap<GrammarId, HashSet<Rc<Grammar<TKind, NTKind>>>>,
  // TODO: token_ast_mapper
}

impl<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Default + Clone + 'static,
    LexerErrorType: 'static,
  > Dfa<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>
{
  pub fn new(
    entry_nts: HashSet<GrammarId>,
    entry_state: Rc<
      State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
    >,
    states: HashMap<
      StateId,
      Rc<State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
    >,
    follow_sets: HashMap<GrammarId, HashSet<Rc<Grammar<TKind, NTKind>>>>,
  ) -> Self {
    Self {
      entry_nts,
      entry_state,
      follow_sets,
      states,
    }
  }

  pub fn parse<'buffer>(
    &self,
    buffer: Vec<ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>>,
    lexer: TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    lexer_panic_handler: &LexerPanicHandler<TKind, LexerActionState, LexerErrorType>,
    global: &Rc<RefCell<Global>>,
  ) -> DfaParseOutput<
    'buffer,
    TKind,
    NTKind,
    ASTData,
    ErrorType,
    Global,
    LexerActionState,
    LexerErrorType,
  > {
    // TODO: move to ParsingState::new()?
    let mut parsing_state = ParsingState {
      buffer,
      state_stack: Stack::new(vec![self.entry_state.clone().into()]),
      reducing_stack: Vec::new(),
      next_token: None,
      lexer,
      need_lex: true, // at the beginning we should lex for a new AST node // TODO: is this true? maybe we want to reduce when we already have nodes in buffer
      errors: Vec::new(),
    };

    loop {
      if parsing_state.need_lex {
        // try to lex a new one
        if parsing_state.try_lex(&self.states, global, lexer_panic_handler) {
          continue;
        }

        // else, reach EOF and can't lex anymore
        // enter parser panic mode
        todo!();
      }

      // else, no need to lex, just try to reduce
      match parsing_state.try_reduce(
        &self.entry_nts,
        &self.follow_sets,
        &self.states,
        global,
        lexer_panic_handler,
      ) {
        TryReduceResult::NeedLex => continue,
        TryReduceResult::EnterPanicMode => todo!(),
        TryReduceResult::Done => {
          return DfaParseOutput {
            lexer: parsing_state.lexer,
            buffer: parsing_state.buffer,
            errors: parsing_state.errors,
          };
        }
      }
    }
  }
}
