pub type ASTNodeFirstMatchSelector<ASTNodeType> =
  Box<dyn Fn(&[ASTNodeType]) -> Option<ASTNodeType>>;
// TODO
