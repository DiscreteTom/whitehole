use super::{
  parsing::{ParsingState, Stack, TryReduceResult},
  state::{State, StateId, StatefulState},
};
use crate::{
  lexer::{token::TokenKind, trimmed::TrimmedLexer},
  parser::{
    ast::ASTNode,
    elr::{
      grammar::grammar::{Grammar, GrammarId},
      parser::ParseContinuable,
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
  pub continuable: ParseContinuable<
    Stack<
      StatefulState<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
    >,
  >,
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

  pub fn entry_state(
    &self,
  ) -> &Rc<State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>> {
    &self.entry_state
  }

  pub fn parse<'buffer>(
    &self,
    buffer: Vec<ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>>,
    state_stack: Stack<
      StatefulState<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
    >,
    reducing_stack: Vec<usize>,
    lexer: TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
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
      state_stack,
      reducing_stack,
      next_token: None,
      lexer,
      need_lex: true, // at the beginning we should lex for a new AST node // TODO: is this true? maybe we want to reduce when we already have nodes in buffer
      errors: Vec::new(),
    };

    loop {
      if parsing_state.need_lex {
        // try to lex a new one
        if parsing_state.try_lex(&self.states, global) {
          continue;
        }

        // TODO: enter panic mode
        todo!()
      }

      // else, no need to lex, just try to reduce
      match parsing_state.try_reduce(&self.entry_nts, &self.follow_sets, &self.states, global) {
        TryReduceResult::NeedLex => continue,
        TryReduceResult::EnterPanicMode => todo!(),
        TryReduceResult::Done { continuable } => {
          return DfaParseOutput {
            lexer: parsing_state.lexer,
            buffer: parsing_state.buffer,
            errors: parsing_state.errors,
            continuable: if continuable {
              ParseContinuable::Yes(parsing_state.state_stack)
            } else {
              ParseContinuable::No
            },
          };
        }
      }
    }
  }
}
