pub struct Stack<T> {
  stack: Vec<T>,
}

impl<T> Clone for Stack<T>
where
  T: Clone,
{
  fn clone(&self) -> Self {
    Self {
      stack: self.stack.clone(),
    }
  }
}

impl<T> Stack<T> {
  pub fn new(stack: Vec<T>) -> Self {
    Self { stack }
  }
  pub fn push(&mut self, item: T) {
    self.stack.push(item);
  }
  pub fn pop(&mut self) -> Option<T> {
    self.stack.pop()
  }
  pub fn current(&self) -> Option<&T> {
    self.stack.last()
  }
  pub fn clear(&mut self) {
    self.stack.clear();
  }
}

pub struct ParsingState<ASTNodeType, StateType, LexerType> {
  pub buffer: Vec<ASTNodeType>,
  /// The index of buffer.
  pub index: usize,
  pub state_stack: Stack<StateType>,
  pub lexer: LexerType,
}
