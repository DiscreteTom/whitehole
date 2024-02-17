use std::collections::HashSet;

#[derive(PartialEq)]
pub enum ConflictKind {
  ReduceShift,
  ReduceReduce,
}

pub struct ConflictCondition<GrammarType> {
  /// A list of T/Literal grammars that will cause conflicts when appear as the next node.
  // [[@next can only be T or Literal]]
  pub next: HashSet<GrammarType>,
  /// Is this a conflict if reaching end of input?
  pub eof: bool,
}

pub struct Conflict<AnotherType, GrammarType> {
  pub kind: ConflictKind,
  /// If this is a R-S conflict, this rule is a shifter rule. If this is a R-R conflict, this rule is a reducer rule.
  pub another: AnotherType,
  pub condition: ConflictCondition<GrammarType>,
}
