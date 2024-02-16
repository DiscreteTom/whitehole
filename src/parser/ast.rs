use super::traverser::Traverser;
use crate::lexer::token::{Range, TokenKind};
use std::{cell::RefCell, rc::Rc};

pub enum ASTNodeKind<
  'buffer,
  TKind: TokenKind<TKind>,
  NTKind: TokenKind<NTKind>,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  T(TKind, &'buffer str),
  NT(NTKind, Traverser<TKind, NTKind, ASTData, ErrorType, Global>),
}

pub struct ASTNode<
  'buffer,
  TKind: TokenKind<TKind>,
  NTKind: TokenKind<NTKind>, // TODO: don't use TokenKind? use another trait?
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  // make these fields public so the user can destruct the struct and get the fields
  pub kind: ASTNodeKind<'buffer, TKind, NTKind, ASTData, ErrorType, Global>,
  pub range: Range,
  pub children: Vec<usize>,
  pub global: Rc<RefCell<Global>>,
  pub data: Option<ASTData>,
  pub error: Option<ErrorType>,
  pub parent: Option<usize>,
}

impl<
    'buffer,
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind>,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  > ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>
{
  pub fn new_t(
    kind: TKind,
    text: &'buffer str,
    range: Range,
    global: Rc<RefCell<Global>>,
    data: Option<ASTData>,
    error: Option<ErrorType>,
  ) -> Self {
    Self {
      kind: ASTNodeKind::T(kind, text),
      range,
      children: Vec::new(),
      global,
      data,
      error,
      // the parent will be set later
      parent: None,
    }
  }

  pub fn new_nt(
    kind: NTKind,
    range: Range,
    children: Vec<usize>,
    global: Rc<RefCell<Global>>,
    data: Option<ASTData>,
    error: Option<ErrorType>,
    traverser: Traverser<TKind, NTKind, ASTData, ErrorType, Global>,
  ) -> Self {
    Self {
      kind: ASTNodeKind::NT(kind, traverser),
      range,
      children,
      global,
      data,
      error,
      // the parent will be set later
      parent: None,
    }
  }

  /// Use the traverser to calculate data and return the data.
  pub fn traverse(&mut self) -> &Option<ASTData> {
    if let ASTNodeKind::NT(_, traverser) = &self.kind {
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
