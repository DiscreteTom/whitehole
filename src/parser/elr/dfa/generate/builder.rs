use crate::{
  lexer::token::TokenKind,
  parser::elr::grammar::{
    grammar::{GrammarId, GrammarKind},
    grammar_rule::GrammarRule,
    grammar_rule_repo::GrammarRuleRepo,
  },
};
use std::{
  collections::{BTreeSet, HashMap, HashSet},
  rc::Rc,
};

use super::{candidate_repo::CandidateRepo, state_repo::StateRepo};

pub fn prepare<
  TKind: TokenKind,
  NTKind: TokenKind + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
>(
  nts: HashSet<GrammarId>, // TODO: don't pass nts because this can be calculated by gr_repo
  entry_nts: HashSet<GrammarId>,
  gr_repo: GrammarRuleRepo<TKind, NTKind, ASTData, ErrorType, Global>,
) {
  let nt_closures = calc_all_nt_closures(&nts, &gr_repo);

  // init all initial candidates, initial candidate is candidate with digested=0
  let mut cs = CandidateRepo::with_initial(gr_repo.grs());

  let entry_candidates = calc_grs_closure(
    gr_repo
      .grs()
      .iter()
      .filter(|gr| entry_nts.contains(&gr.nt().id()))
      .map(|gr| gr.clone())
      .collect(),
    &gr_repo,
  )
  .iter()
  .map(|gr| cs.get_initial(gr.id()).id().clone())
  .collect::<BTreeSet<_>>();

  let mut state_repo = StateRepo::with_entry(entry_candidates);
  state_repo.calc_all_states(&get_all_grammar_id(&gr_repo), &mut cs, &nt_closures);

  let first_sets = calc_first_sets(&nt_closures);
  let follow_sets = calc_follow_sets(&gr_repo, &first_sets);
}

/// If a rule starts with an NT, merge result with that NT's grammar rules.
///
/// E.g. knowing `A := B 'c'` and `B := 'd'`, we can infer `A := 'd' 'c'`.
/// When we construct DFA state, if a state has the candidate `A := # B 'c'`,
/// it should also have the candidate `B := # 'd'`.
// TODO: just return id?
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

/// Calculate all direct/indirect grammar rules which can reduce to the specified NT.
///
/// E.g. knowing `A := B 'c'` and `B := 'd'`, we can infer `A := 'd' 'c'`.
/// When we construct DFA state, if a state has the candidate `X := # A`,
/// it should also have the candidate `A := # B 'c'` and `B := # 'd'`.
/// In this case, `A := # B 'c'` and `B := # 'd'` are the closure of the NT 'A'.
fn calc_nt_closure<
  TKind: TokenKind,
  NTKind: TokenKind + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
>(
  nt_id: &GrammarId,
  gr_repo: &GrammarRuleRepo<TKind, NTKind, ASTData, ErrorType, Global>,
) -> Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>> {
  calc_grs_closure(
    gr_repo
      .grs()
      .iter()
      .filter(|gr| gr.nt().id() == *nt_id)
      .map(|gr| gr.clone())
      .collect(),
    gr_repo,
  )
}

fn calc_all_nt_closures<
  TKind: TokenKind,
  NTKind: TokenKind + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
>(
  nt_ids: &HashSet<GrammarId>,
  gr_repo: &GrammarRuleRepo<TKind, NTKind, ASTData, ErrorType, Global>,
) -> HashMap<GrammarId, Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>>> {
  nt_ids
    .iter()
    .map(|nt_id| (nt_id.clone(), calc_nt_closure(nt_id, gr_repo)))
    .collect::<HashMap<_, _>>()
}

fn get_all_grammar_id<
  TKind: TokenKind,
  NTKind: TokenKind + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
>(
  gr_repo: &GrammarRuleRepo<TKind, NTKind, ASTData, ErrorType, Global>,
) -> HashSet<GrammarId> {
  // collect all grammars in grammar rules only,
  // don't collect grammar rules' NTs, because
  // some NTs might not appear in grammar rules (entry-only NTs).
  // when entry-only NTs appear in parser's buffer (e.g. user provided buffer when parse),
  // the `parser.parse` should throw StateCacheMissError.
  // if we do collect entry-only NTs,
  // the `parser.parse` will just reject the input without throwing StateCacheMissError.
  // TODO: update comments above
  gr_repo
    .grs()
    .iter()
    .map(|gr| gr.rule().iter().map(|g| g.id().clone()).collect::<Vec<_>>())
    .flat_map(|ids| ids.into_iter())
    .collect()
}

fn calc_first_sets<
  TKind: TokenKind,
  NTKind: TokenKind + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
>(
  nt_closures: &HashMap<GrammarId, Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>>>>,
) -> HashMap<GrammarId, HashSet<GrammarId>> {
  let mut result = HashMap::new();

  nt_closures.iter().for_each(|(nt, grs)| {
    let grammar_set = result.entry(nt.clone()).or_insert_with(|| HashSet::new());
    // for each direct/indirect grammar rule, add first grammar to first set
    // including T and NT
    grs.iter().for_each(|gr| {
      grammar_set.insert(gr.rule()[0].kind().id());
    });
  });

  result
}

fn calc_follow_sets<
  TKind: TokenKind,
  NTKind: TokenKind + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
>(
  gr_repo: &GrammarRuleRepo<TKind, NTKind, ASTData, ErrorType, Global>,
  first_sets: &HashMap<GrammarId, HashSet<GrammarId>>,
) -> HashMap<GrammarId, HashSet<GrammarId>> {
  let mut result = HashMap::new();

  gr_repo.grs().iter().for_each(|gr| {
    gr.rule().iter().enumerate().for_each(|(i, g)| {
      if i < gr.rule().len() - 1 {
        let next_grammar = &gr.rule()[i + 1];
        // next grammar exists, merge the current grammar's follow set with next grammar
        let grammar_set = result
          .entry(g.kind().id())
          .or_insert_with(|| HashSet::new());
        grammar_set.insert(next_grammar.kind().id());
        // if next grammar is also NT, merge with its first set
        if let GrammarKind::NT(nt) = next_grammar.kind() {
          // every NT should have a first set, so we can unwrap the result
          grammar_set.extend(first_sets.get(&nt.id()).unwrap());
        }
      }
    });
  });

  // the last grammar's follow set should merge with the target NT's follow set
  // be ware: don't merge the target NT's follow set with the last grammar's follow set
  // the last grammar's follow set should be a super set of the target NT's follow set, not vice versa
  loop {
    let mut changed = false;

    gr_repo.grs().iter().for_each(|gr| {
      let last_grammar = gr.rule().last().unwrap();
      let nt_follow = result.get(&gr.nt().id()).unwrap().clone(); // TODO: prevent the clone
      let last_grammar_follow = result.get_mut(&last_grammar.id()).unwrap();
      let len = last_grammar_follow.len();
      last_grammar_follow.extend(nt_follow);
      if last_grammar_follow.len() != len {
        changed = true;
      }
    });

    if !changed {
      break;
    }
  }

  result
}
