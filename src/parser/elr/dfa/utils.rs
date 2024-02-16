use crate::{
  lexer::{
    expectation::Expectation,
    token::{TokenKind, TokenKindId},
    trimmed::TrimmedLexer,
  },
  parser::ast::ASTNode,
};
use std::{cell::RefCell, rc::Rc};

pub struct LexGrammarOutput<'buffer, TKind, NodeType, LexerType> {
  pub t_kind_id: TokenKindId<TKind>,
  pub text: &'buffer str,
  pub node: NodeType,
  pub lexer: LexerType,
}

pub fn lex_grammar<
  'buffer,
  TKind: TokenKind<TKind>,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
>(
  expectation: Expectation<TKind>,
  lexer: &TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
  global: &Rc<RefCell<Global>>,
) -> Option<
  LexGrammarOutput<
    'buffer,
    TKind,
    ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>,
    TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
  >,
> {
  // because of re-lex, we may store many lexers
  // so we clone the lexer to prevent side effect.
  // we must clone the lexer here to prevent unnecessary clone.
  // you may think using peek is more efficient, but it's not,
  // since we still need to clone and store the new lexer state and action state
  // so it's actually the same.
  // TODO: don't clone the lexer if we disable re-lex or when re-lex won't happen
  let lexer = lexer.clone();

  let res = lexer.lex_expect(expectation);
  res.token.and_then(move |token| {
    // TODO: set node data
    Some(LexGrammarOutput {
      t_kind_id: token.kind.id(),
      text: token.content,
      node: ASTNode::new_t(
        token.kind,
        token.content,
        token.range,
        global.clone(),
        None,
        None,
      ),
      lexer: res.lexer.into(), // trim the lexer and convert into TrimmedLexer
    })
  })
}
