use crate::parser::elr::grammar::grammar::GrammarId;
use std::collections::HashSet;

#[derive(PartialEq)]
pub enum ConflictKind {
  ReduceShift,
  ReduceReduce,
}

pub struct ConflictCondition {
  /// A list of grammars that will cause conflicts when appear as the next node.
  pub next: HashSet<GrammarId>,
  /// Is this a conflict if reaching end of input?
  pub eof: bool,
}

pub struct Conflict<AnotherType> {
  pub kind: ConflictKind,
  /// If this is a R-S conflict, this rule is a shifter rule. If this is a R-R conflict, this rule is a reducer rule.
  pub another: AnotherType,
  pub condition: ConflictCondition,
}
