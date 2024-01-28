use crate::lexer::token::{Range, TokenKind};
use std::{borrow::BorrowMut, cell::RefCell, rc::Rc};

macro_rules! impl_ast_node {
  () => {
    pub fn buffer(&self) -> &'ast_buffer [ASTNode<TKind, NTKind, ASTData, ErrorType, Global>] {
      self.buffer
    }
    pub fn range(&self) -> &Range {
      &self.range
    }
    pub fn global(&self) -> &Rc<RefCell<Global>> {
      &self.global
    }
    pub fn global_mut(&mut self) -> &mut Rc<RefCell<Global>> {
      self.global.borrow_mut()
    }
    pub fn data(&self) -> &Option<ASTData> {
      &self.data
    }
    pub fn data_mut(&mut self) -> &mut Option<ASTData> {
      self.data.borrow_mut()
    }
    pub fn error(&self) -> &Option<ErrorType> {
      &self.error
    }
    pub fn error_mut(&mut self) -> &mut Option<ErrorType> {
      self.error.borrow_mut()
    }
    pub fn parent(&self) -> &Option<usize> {
      &self.parent
    }
    pub fn parent_mut(&mut self) -> &mut Option<usize> {
      self.parent.borrow_mut()
    }
    pub fn parent_node(&self) -> Option<&ASTNode<TKind, NTKind, ASTData, ErrorType, Global>> {
      self.parent.map(|i| &self.buffer[i])
    }
  };
}

pub struct TNode<
  'ast_buffer,
  TKind: TokenKind,
  NTKind,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  kind: TKind,

  buffer: &'ast_buffer [ASTNode<'ast_buffer, TKind, NTKind, ASTData, ErrorType, Global>],
  range: Range,
  global: Rc<RefCell<Global>>,
  data: Option<ASTData>,
  error: Option<ErrorType>,
  parent: Option<usize>,
}

impl<
    'ast_buffer,
    TKind: TokenKind,
    NTKind,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  > TNode<'ast_buffer, TKind, NTKind, ASTData, ErrorType, Global>
{
  pub fn new(
    kind: TKind,
    buffer: &'ast_buffer [ASTNode<TKind, NTKind, ASTData, ErrorType, Global>],
    range: Range,
    data: Option<ASTData>,
    error: Option<ErrorType>,
    parent: Option<usize>,
    global: Rc<RefCell<Global>>,
  ) -> Self {
    TNode {
      buffer,
      kind,
      range,
      data,
      error,
      parent,
      global,
    }
  }

  pub fn kind(&self) -> &TKind {
    &self.kind
  }

  impl_ast_node!();

  pub fn traverse(&self) -> &Option<ASTData> {
    // for T nodes, data should be set by the user
    // when transforming a token into a T node
    // so the traverse function won't calculate data
    // just return the data
    &self.data
  }
}

pub struct NTNode<
  'ast_buffer,
  TKind: TokenKind,
  NTKind,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  kind: NTKind,
  children: Vec<usize>,
  traverser: Box<dyn Fn(&NTNode<TKind, NTKind, ASTData, ErrorType, Global>) -> Option<ASTData>>,

  buffer: &'ast_buffer [ASTNode<'ast_buffer, TKind, NTKind, ASTData, ErrorType, Global>],
  range: Range,
  global: Rc<RefCell<Global>>,
  data: Option<ASTData>,
  error: Option<ErrorType>,
  parent: Option<usize>,
}

impl<
    'ast_buffer,
    TKind: TokenKind,
    NTKind,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  > NTNode<'ast_buffer, TKind, NTKind, ASTData, ErrorType, Global>
{
  pub fn kind(&self) -> &NTKind {
    &self.kind
  }

  impl_ast_node!();

  pub fn children(&self) -> &Vec<usize> {
    &self.children
  }
  pub fn children_nodes(
    &self,
  ) -> impl Iterator<Item = &ASTNode<TKind, NTKind, ASTData, ErrorType, Global>> {
    self.children.iter().map(move |i| &self.buffer[*i])
  }

  /// Use the traverser to calculate data and return the data.
  pub fn traverse(&mut self) -> &Option<ASTData> {
    self.data = (self.traverser)(&self); // TODO: should be mut self?
    &self.data
  }

  // pub fn first(&self) -> Option<ASTNode<TKind, NTKind, ASTData, ErrorType, Global>> {
  // TODO
  // }
}

pub enum ASTNode<
  'ast_buffer,
  TKind: TokenKind,
  NTKind,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  T(TNode<'ast_buffer, TKind, NTKind, ASTData, ErrorType, Global>),
  NT(NTNode<'ast_buffer, TKind, NTKind, ASTData, ErrorType, Global>),
}

// TODO: any idea to avoid this?
impl<
    'ast_buffer,
    TKind: TokenKind,
    NTKind,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  > ASTNode<'ast_buffer, TKind, NTKind, ASTData, ErrorType, Global>
{
  pub fn buffer(&self) -> &'ast_buffer [ASTNode<TKind, NTKind, ASTData, ErrorType, Global>] {
    match self {
      ASTNode::T(node) => node.buffer(),
      ASTNode::NT(node) => node.buffer(),
    }
  }
  pub fn range(&self) -> &Range {
    match self {
      ASTNode::T(node) => node.range(),
      ASTNode::NT(node) => node.range(),
    }
  }
  pub fn global(&self) -> &Rc<RefCell<Global>> {
    match self {
      ASTNode::T(node) => node.global(),
      ASTNode::NT(node) => node.global(),
    }
  }
  pub fn global_mut(&mut self) -> &mut Rc<RefCell<Global>> {
    match self {
      ASTNode::T(node) => node.global_mut(),
      ASTNode::NT(node) => node.global_mut(),
    }
  }
  pub fn data(&self) -> &Option<ASTData> {
    match self {
      ASTNode::T(node) => node.data(),
      ASTNode::NT(node) => node.data(),
    }
  }
  pub fn data_mut(&mut self) -> &mut Option<ASTData> {
    match self {
      ASTNode::T(node) => node.data_mut(),
      ASTNode::NT(node) => node.data_mut(),
    }
  }
  pub fn error(&self) -> &Option<ErrorType> {
    match self {
      ASTNode::T(node) => node.error(),
      ASTNode::NT(node) => node.error(),
    }
  }
  pub fn error_mut(&mut self) -> &mut Option<ErrorType> {
    match self {
      ASTNode::T(node) => node.error_mut(),
      ASTNode::NT(node) => node.error_mut(),
    }
  }
  pub fn parent(&self) -> &Option<usize> {
    match self {
      ASTNode::T(node) => node.parent(),
      ASTNode::NT(node) => node.parent(),
    }
  }
  pub fn parent_mut(&mut self) -> &mut Option<usize> {
    match self {
      ASTNode::T(node) => node.parent_mut(),
      ASTNode::NT(node) => node.parent_mut(),
    }
  }
  pub fn parent_node(&self) -> Option<&ASTNode<TKind, NTKind, ASTData, ErrorType, Global>> {
    match self {
      ASTNode::T(node) => node.parent_node(),
      ASTNode::NT(node) => node.parent_node(),
    }
  }

  pub fn traverse(&mut self) -> &Option<ASTData> {
    match self {
      ASTNode::T(node) => node.traverse(),
      ASTNode::NT(node) => node.traverse(),
    }
  }
  pub fn children(&self) -> Option<&Vec<usize>> {
    match self {
      ASTNode::T(_) => None,
      ASTNode::NT(node) => Some(node.children()),
    }
  }
  pub fn children_nodes(
    &self,
  ) -> Option<impl Iterator<Item = &ASTNode<TKind, NTKind, ASTData, ErrorType, Global>>> {
    match self {
      ASTNode::T(_) => None,
      ASTNode::NT(node) => Some(node.children_nodes()),
    }
  }
}

impl<
    'ast_buffer,
    TKind: TokenKind,
    NTKind,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  > From<TNode<'ast_buffer, TKind, NTKind, ASTData, ErrorType, Global>>
  for ASTNode<'ast_buffer, TKind, NTKind, ASTData, ErrorType, Global>
{
  fn from(node: TNode<'ast_buffer, TKind, NTKind, ASTData, ErrorType, Global>) -> Self {
    ASTNode::T(node)
  }
}

impl<
    'ast_buffer,
    TKind: TokenKind,
    NTKind,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  > From<NTNode<'ast_buffer, TKind, NTKind, ASTData, ErrorType, Global>>
  for ASTNode<'ast_buffer, TKind, NTKind, ASTData, ErrorType, Global>
{
  fn from(node: NTNode<'ast_buffer, TKind, NTKind, ASTData, ErrorType, Global>) -> Self {
    ASTNode::NT(node)
  }
}
