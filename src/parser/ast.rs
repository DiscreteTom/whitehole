use crate::lexer::token::{Range, TokenKind};
use std::{cell::RefCell, rc::Rc};

pub enum ASTNodeType<Kind: TokenKind, ASTData: 'static, ErrorType: 'static, Global: 'static> {
  T,
  NT {
    /// Traverser should calculate the node's data.
    traverser: Box<dyn Fn(&ASTNode<Kind, ASTData, ErrorType, Global>) -> Option<ASTData>>,
  },
}

pub struct ASTNode<Kind: TokenKind, ASTData: 'static, ErrorType: 'static, Global: 'static> {
  pub kind: Kind,
  pub range: Range,
  pub children: Vec<usize>,
  pub global: Rc<RefCell<Global>>,
  pub data: Option<ASTData>,
  pub error: Option<ErrorType>,
  pub parent: Option<usize>,

  // hide this so the user can only call `traverse`
  node_type: ASTNodeType<Kind, ASTData, ErrorType, Global>,
}

impl<Kind: TokenKind, ASTData: 'static, ErrorType: 'static, Global: 'static>
  ASTNode<Kind, ASTData, ErrorType, Global>
{
  pub fn new_t(
    kind: Kind,
    range: Range,
    global: Rc<RefCell<Global>>,
    data: Option<ASTData>,
    error: Option<ErrorType>,
  ) -> Self {
    Self {
      kind,
      range,
      children: Vec::new(),
      global,
      data,
      error,
      // the parent will be set later
      parent: None,
      node_type: ASTNodeType::T,
    }
  }

  pub fn new_nt(
    kind: Kind,
    range: Range,
    children: Vec<usize>,
    global: Rc<RefCell<Global>>,
    data: Option<ASTData>,
    error: Option<ErrorType>,
    traverser: Box<dyn Fn(&ASTNode<Kind, ASTData, ErrorType, Global>) -> Option<ASTData>>,
  ) -> Self {
    Self {
      kind,
      range,
      children,
      global,
      data,
      error,
      // the parent will be set later
      parent: None,
      node_type: ASTNodeType::NT { traverser },
    }
  }

  pub fn is_t(&self) -> bool {
    matches!(self.node_type, ASTNodeType::T)
  }

  /// Use the traverser to calculate data and return the data.
  pub fn traverse(&mut self) -> &Option<ASTData> {
    if let ASTNodeType::NT { traverser } = &self.node_type {
      self.data = (traverser)(&self); // TODO: should be mut self?
    }
    // for T nodes, data should be set by the user
    // when transforming a token into a T node
    // so the traverse function won't calculate data
    // just return the data

    &self.data
  }

  // pub fn first(&self) -> Option<ASTNode<Kind, ASTData, ErrorType, Global>> {
  // TODO
  // }
}
