use super::{
  parsing::{ParsingState, Stack, TryReduceResult},
  state::{State, StateId},
};
use crate::{
  lexer::{
    token::{TokenKind, TokenKindId},
    trimmed::TrimmedLexer,
  },
  parser::{
    ast::ASTNode,
    elr::{grammar::grammar::GrammarId, parser::ParseContinuable},
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
  pub buffer: Vec<ASTNode<TKind, NTKind, ASTData, ErrorType, Global>>,
  pub errors: Vec<usize>,
  pub continuable: ParseContinuable<
    Stack<Rc<State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>>,
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
  entry_nts: HashSet<TokenKindId<NTKind>>,
  entry_state:
    Rc<State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
  states: HashMap<
    StateId,
    Rc<State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
  >,
  follow_sets: HashMap<GrammarId, HashSet<GrammarId>>,
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
    entry_nts: HashSet<TokenKindId<NTKind>>,
    entry_state: Rc<
      State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
    >,
    states: HashMap<
      StateId,
      Rc<State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
    >,
    follow_sets: HashMap<GrammarId, HashSet<GrammarId>>,
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
    buffer: Vec<ASTNode<TKind, NTKind, ASTData, ErrorType, Global>>,
    state_stack: Stack<
      Rc<State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
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
      lexer,
      need_lex: true, // at the beginning we should lex for a new AST node // TODO: is this true? maybe we want to reduce when we already have nodes in buffer
      try_lex_index: 0, // from the first candidate
      lexed_grammars: HashSet::new(), // no grammars are lexed at the beginning
      lexed_without_expectation: false, // non-expectational lex is not done at the beginning
      errors: Vec::new(),
    };

    loop {
      if parsing_state.need_lex {
        // try to lex a new one
        if parsing_state.try_lex(&self.states, global) {
          continue;
        } else {
          // TODO: enter panic mode
          todo!()
        }
      }

      // else, no need to lex, just try to reduce
      match parsing_state.try_reduce(&self.entry_nts, &self.follow_sets, &self.states) {
        TryReduceResult::NeedLex => continue,
        TryReduceResult::EnterPanicMode => todo!(),
        TryReduceResult::Done(continuable) => {
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
