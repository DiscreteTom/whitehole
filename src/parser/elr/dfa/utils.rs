use crate::{
  lexer::{
    expectation::Expectation,
    token::{TokenKind, TokenKindId},
    trimmed::TrimmedLexer,
  },
  parser::ast::ASTNode,
};
use std::{cell::RefCell, rc::Rc};

pub struct LexGrammarOutput<'buffer, TKind, NodeType> {
  pub t_kind_id: TokenKindId<TKind>,
  pub text: &'buffer str,
  pub node: NodeType,
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
  lexer: &mut TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
  global: &Rc<RefCell<Global>>,
) -> Option<
  LexGrammarOutput<'buffer, TKind, ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>>,
> {
  let (res, _) = lexer.lex_expect(expectation);
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
    })
  })
}
