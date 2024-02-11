use super::ast::ASTNode;
use crate::lexer::token::TokenKind;
use std::rc::Rc;

pub type Traverser<
  TKind: TokenKind<TKind>,
  NTKind: TokenKind<NTKind>,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> = Rc<dyn Fn(&ASTNode<TKind, NTKind, ASTData, ErrorType, Global>) -> Option<ASTData>>;
