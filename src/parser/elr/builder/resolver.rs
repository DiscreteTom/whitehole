use super::conflict::ConflictKind;
use crate::parser::elr::grammar::grammar::GrammarId;
use std::collections::HashSet;

pub enum ResolvedConflictConditionNext {
  Any,
  Some(HashSet<GrammarId>),
}

pub struct ResolvedConflictCondition {
  pub next: ResolvedConflictConditionNext,
  pub eof: bool,
}

pub struct ResolvedConflict<GrammarRuleType, AccepterType> {
  pub kind: ConflictKind,
  /// If this is a R-S conflict, this rule is a shifter rule. If this is a R-R conflict, this rule is a reducer rule.
  pub another_rule: GrammarRuleType,
  pub accepter: AccepterType,
  pub condition: ResolvedConflictCondition,
}
