use crate::{
  lexer::token::TokenKind,
  parser::elr::grammar::{
    grammar::GrammarKind, grammar_rule::GrammarRule, grammar_rule_repo::GrammarRuleRepo,
  },
};
use std::{
  collections::{HashMap, HashSet},
  rc::Rc,
};

use super::candidate_repo::CandidateRepo;

pub fn prepare<
  TKind: TokenKind,
  NTKind: TokenKind + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
>(
  gr_repo: GrammarRuleRepo<TKind, NTKind, ASTData, ErrorType, Global>,
) {
  // init all initial candidates, initial candidate is candidate with digested=0
  let cs = CandidateRepo::with_initial(gr_repo.grs());
}

/// If a rule starts with an NT, merge result with that NT's grammar rules.
///
/// E.g. knowing `A := B 'c'` and `B := 'd'`, we can infer `A := 'd' 'c'`.
/// When we construct DFA state, if a state has the candidate `A := # B 'c'`,
/// it should also have the candidate `B := # 'd'`.
fn calc_grs_closure<
  TKind: TokenKind,
  NTKind: TokenKind + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
>(
  grs: Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>>,
  gr_repo: &GrammarRuleRepo<TKind, NTKind, ASTData, ErrorType, Global>,
) -> Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>> {
  // init result as a hashmap with given grammar rules
  // so we can check if a grammar rule is already in the result by id
  let mut result = grs
    .into_iter()
    .map(|gr| (gr.id().clone(), gr))
    .collect::<HashMap<_, _>>();

  // at the beginning, treat all given grammar rules as unexpanded
  // we use hashmap to we can just swap this and next_unexpanded later
  let mut unexpanded = result.clone();

  loop {
    // keep track of next unexpanded grammar rules
    // we use hashmap to check if a grammar rule is already in this by id
    // and store the grammar rule as value
    let mut next_unexpanded = HashMap::new();

    unexpanded.iter().for_each(|(_, gr)| {
      // only expand if the unexpanded grammar rule's first grammar is NT
      if let GrammarKind::NT(nt) = gr.rule()[0].kind() {
        gr_repo
          .grs()
          .iter()
          // find all grammar rules that yields the NT
          .filter(|gr2| gr2.nt().id() == nt.id())
          .for_each(|gr2| {
            // no need to check if the gr2 is already in `unexpanded`
            // since unexpanded grammar rules are already in the result
            // so we just make sure the gr2 is not already in the result and next_unexpanded
            if !result.contains_key(gr2.id()) && !next_unexpanded.contains_key(gr2.id()) {
              next_unexpanded.insert(gr2.id().clone(), gr2.clone());
            }
          });
      }
    });

    if next_unexpanded.len() == 0 {
      // done
      break;
    }

    // append next_unexpanded to result
    for (id, gr) in &next_unexpanded {
      result.insert(id.clone(), gr.clone());
    }
    // and swap unexpanded with next_unexpanded
    unexpanded = next_unexpanded;
  }

  // convert back to Vec
  result.into_iter().map(|(_, gr)| gr).collect()
}
