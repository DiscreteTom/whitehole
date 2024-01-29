use crate::lexer::token::{Range, TokenKind};
use std::{borrow::BorrowMut, cell::RefCell, rc::Rc};

macro_rules! impl_ast_node {
  () => {
    pub fn range(&self) -> &Range {
      &self.range
    }
    pub fn global(&self) -> &Rc<RefCell<Global>> {
      &self.global
    }
    // don't return `&mut Rc<RefCell<Global>>` because we only want to mutate the content
    // TODO
    // pub fn global_mut(&mut self) -> &mut RefCell<Global> {
    //   self.global.borrow_mut()
    // }
    pub fn data(&self) -> &Option<ASTData> {
      &self.data
    }
    pub fn data_mut(&mut self) -> &mut Option<ASTData> {
      &mut self.data
    }
    pub fn error(&self) -> &Option<ErrorType> {
      &self.error
    }
    pub fn error_mut(&mut self) -> &mut Option<ErrorType> {
      self.error.borrow_mut()
    }
    pub fn parent(&self) -> &Option<Rc<NTNode<TKind, NTKind, ASTData, ErrorType, Global>>> {
      &self.parent
    }
    pub fn parent_mut(
      &mut self,
    ) -> &mut Option<Rc<NTNode<TKind, NTKind, ASTData, ErrorType, Global>>> {
      &mut self.parent
    }
  };
}

pub struct TNode<TKind: TokenKind, NTKind, ASTData: 'static, ErrorType: 'static, Global: 'static> {
  kind: TKind,

  range: Range,
  global: Rc<RefCell<Global>>,
  data: Option<ASTData>,
  error: Option<ErrorType>,
  parent: Option<Rc<NTNode<TKind, NTKind, ASTData, ErrorType, Global>>>,
}

impl<TKind: TokenKind, NTKind, ASTData: 'static, ErrorType: 'static, Global: 'static>
  TNode<TKind, NTKind, ASTData, ErrorType, Global>
{
  pub fn new(
    kind: TKind,
    range: Range,
    data: Option<ASTData>,
    error: Option<ErrorType>,
    parent: Option<Rc<NTNode<TKind, NTKind, ASTData, ErrorType, Global>>>,
    global: Rc<RefCell<Global>>,
  ) -> Self {
    TNode {
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

pub struct NTNode<TKind: TokenKind, NTKind, ASTData: 'static, ErrorType: 'static, Global: 'static> {
  kind: NTKind,
  children: Vec<Rc<NTNode<TKind, NTKind, ASTData, ErrorType, Global>>>,
  traverser: Box<dyn Fn(&NTNode<TKind, NTKind, ASTData, ErrorType, Global>) -> Option<ASTData>>,

  range: Range,
  global: Rc<RefCell<Global>>,
  data: Option<ASTData>,
  error: Option<ErrorType>,
  parent: Option<Rc<NTNode<TKind, NTKind, ASTData, ErrorType, Global>>>,
}

impl<TKind: TokenKind, NTKind, ASTData: 'static, ErrorType: 'static, Global: 'static>
  NTNode<TKind, NTKind, ASTData, ErrorType, Global>
{
  pub fn kind(&self) -> &NTKind {
    &self.kind
  }

  impl_ast_node!();

  pub fn children(&self) -> &Vec<Rc<NTNode<TKind, NTKind, ASTData, ErrorType, Global>>> {
    &self.children
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

pub enum ASTNode<TKind: TokenKind, NTKind, ASTData: 'static, ErrorType: 'static, Global: 'static> {
  T(TNode<TKind, NTKind, ASTData, ErrorType, Global>),
  NT(NTNode<TKind, NTKind, ASTData, ErrorType, Global>),
}

// TODO: any idea to avoid this?
impl<TKind: TokenKind, NTKind, ASTData: 'static, ErrorType: 'static, Global: 'static>
  ASTNode<TKind, NTKind, ASTData, ErrorType, Global>
{
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
  // pub fn global_mut(&mut self) -> &RefCell<Global> {
  //   match self {
  //     ASTNode::T(node) => node.global_mut(),
  //     ASTNode::NT(node) => node.global_mut(),
  //   }
  // }
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
  pub fn parent(&self) -> &Option<Rc<NTNode<TKind, NTKind, ASTData, ErrorType, Global>>> {
    match self {
      ASTNode::T(node) => node.parent(),
      ASTNode::NT(node) => node.parent(),
    }
  }
  pub fn parent_mut(
    &mut self,
  ) -> &mut Option<Rc<NTNode<TKind, NTKind, ASTData, ErrorType, Global>>> {
    match self {
      ASTNode::T(node) => node.parent_mut(),
      ASTNode::NT(node) => node.parent_mut(),
    }
  }

  pub fn traverse(&mut self) -> &Option<ASTData> {
    match self {
      ASTNode::T(node) => node.traverse(),
      ASTNode::NT(node) => node.traverse(),
    }
  }
  pub fn children(&self) -> Option<&Vec<Rc<NTNode<TKind, NTKind, ASTData, ErrorType, Global>>>> {
    match self {
      ASTNode::T(_) => None,
      ASTNode::NT(node) => Some(node.children()),
    }
  }
}

impl<TKind: TokenKind, NTKind, ASTData: 'static, ErrorType: 'static, Global: 'static>
  From<TNode<TKind, NTKind, ASTData, ErrorType, Global>>
  for ASTNode<TKind, NTKind, ASTData, ErrorType, Global>
{
  fn from(node: TNode<TKind, NTKind, ASTData, ErrorType, Global>) -> Self {
    ASTNode::T(node)
  }
}

impl<TKind: TokenKind, NTKind, ASTData: 'static, ErrorType: 'static, Global: 'static>
  From<NTNode<TKind, NTKind, ASTData, ErrorType, Global>>
  for ASTNode<TKind, NTKind, ASTData, ErrorType, Global>
{
  fn from(node: NTNode<TKind, NTKind, ASTData, ErrorType, Global>) -> Self {
    ASTNode::NT(node)
  }
}
