use super::ast::ASTNode;
use crate::lexer::token::TokenKind;
use std::rc::Rc;

pub type Traverser<Kind: TokenKind, ASTData: 'static, ErrorType: 'static, Global: 'static> =
  Rc<dyn Fn(&ASTNode<Kind, ASTData, ErrorType, Global>) -> Option<ASTData>>;
