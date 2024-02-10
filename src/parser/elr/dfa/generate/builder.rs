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

fn calc_gr_closure<
  TKind: TokenKind,
  NTKind: TokenKind + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
>(
  grs: Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>>,
  gr_repo: GrammarRuleRepo<TKind, NTKind, ASTData, ErrorType, Global>,
) -> Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>> {
  let mut result = grs;
  let mut appended_gr_ids = result
    .iter()
    .map(|gr| gr.id().clone())
    .collect::<HashSet<_>>();
  let mut done_grs = HashSet::new();

  loop {
    let mut to_be_appended = HashMap::new();

    result.iter().for_each(|gr| {
      if let GrammarKind::NT(nt) = gr.rule()[0].kind() {
        if done_grs.contains(gr.id()) {
          return;
        }

        // mark this gr as done
        done_grs.insert(gr.id().clone());

        gr_repo
          .grs()
          .iter()
          .filter(|gr2| gr2.nt().id() == nt.id())
          .for_each(|gr2| {
            if !appended_gr_ids.contains(gr2.id()) && !to_be_appended.contains_key(gr2.id()) {
              to_be_appended.insert(gr2.id(), gr2.clone());
            }
          });
      }
    });

    if to_be_appended.len() == 0 {
      break;
    }

    for (id, gr) in to_be_appended {
      appended_gr_ids.insert(id.clone());
      result.push(gr);
    }
  }

  result
}
