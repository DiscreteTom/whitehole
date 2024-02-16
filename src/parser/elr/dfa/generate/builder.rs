use super::{
  candidate_repo::CandidateRepo, grammar_rule_repo::GrammarRuleRepo, state_repo::StateRepo,
};
use crate::{
  lexer::token::{TokenKind, TokenKindId},
  parser::elr::{
    dfa::{dfa::Dfa, state::StateId},
    grammar::{
      grammar::{GrammarId, GrammarKind},
      grammar_rule::GrammarRule,
    },
  },
};
use std::{
  collections::{BTreeSet, HashMap, HashSet},
  rc::Rc,
};

pub fn build_dfa<
  TKind: TokenKind<TKind>,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
>(
  entry_nts: HashSet<TokenKindId<NTKind>>,
  gr_repo: GrammarRuleRepo<
    TKind,
    NTKind,
    ASTData,
    ErrorType,
    Global,
    LexerActionState,
    LexerErrorType,
  >,
) -> Dfa<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType> {
  let nts = gr_repo
    .grs()
    .iter()
    .map(|gr| gr.nt().id().clone())
    .collect();

  let nt_closures = calc_all_nt_closures(&nts, &gr_repo);

  // init all initial candidates, initial candidate is candidate with digested=0
  let mut cs = CandidateRepo::with_initial(gr_repo.grs());

  let entry_candidates = calc_grs_closure(
    gr_repo
      .grs()
      .iter()
      .filter(|gr| {
        entry_nts.contains(&match &gr.nt().kind() {
          GrammarKind::NT(nt) => nt.id(),
          _ => unreachable!("GrammarRule's NT must be NTKind"),
        })
      })
      .map(|gr| gr.clone())
      .collect(),
    &gr_repo,
  )
  .iter()
  .map(|gr| cs.get_initial(gr.id()).id().clone())
  .collect::<BTreeSet<_>>();

  let mut state_repo = StateRepo::with_entry(entry_candidates);
  state_repo.calc_all_states(
    &get_all_grammar_id_from_rules(&gr_repo),
    &mut cs,
    &nt_closures,
  );

  let follow_sets = calc_follow_sets(&gr_repo, &calc_first_sets(&nt_closures));

  // convert raw candidates/states to candidates/states
  let candidates = cs.into_candidates();
  let states = state_repo.into_states(&candidates);

  Dfa::new(entry_nts, states[&StateId(0)].clone(), states, follow_sets)
}

/// If a rule starts with an NT, merge result with that NT's grammar rules.
///
/// E.g. knowing `A := B 'c'` and `B := 'd'`, we can infer `A := 'd' 'c'`.
/// When we construct DFA state, if a state has the candidate `A := # B 'c'`,
/// it should also have the candidate `B := # 'd'`.
fn calc_grs_closure<
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
>(
  grs: Vec<
    Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
  >,
  gr_repo: &GrammarRuleRepo<
    TKind,
    NTKind,
    ASTData,
    ErrorType,
    Global,
    LexerActionState,
    LexerErrorType,
  >,
) -> Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>>
{
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
      let first = &gr.rule()[0];
      if let GrammarKind::NT(_) = first.kind() {
        gr_repo
          .grs()
          .iter()
          // find all grammar rules that yields the NT
          .filter(|gr2| gr2.nt().id() == first.id())
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
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
>(
  nt_id: &GrammarId,
  gr_repo: &GrammarRuleRepo<
    TKind,
    NTKind,
    ASTData,
    ErrorType,
    Global,
    LexerActionState,
    LexerErrorType,
  >,
) -> Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>>
{
  calc_grs_closure(
    gr_repo
      .grs()
      .iter()
      .filter(|gr| gr.nt().id() == nt_id)
      .map(|gr| gr.clone())
      .collect(),
    gr_repo,
  )
}

fn calc_all_nt_closures<
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
>(
  nt_ids: &HashSet<GrammarId>,
  gr_repo: &GrammarRuleRepo<
    TKind,
    NTKind,
    ASTData,
    ErrorType,
    Global,
    LexerActionState,
    LexerErrorType,
  >,
) -> HashMap<
  GrammarId,
  Vec<Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>>,
> {
  nt_ids
    .iter()
    .map(|nt_id| (nt_id.clone(), calc_nt_closure(nt_id, gr_repo)))
    .collect::<HashMap<_, _>>()
}

// [[get_all_grammar_id_from_rules]]
fn get_all_grammar_id_from_rules<
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
>(
  gr_repo: &GrammarRuleRepo<
    TKind,
    NTKind,
    ASTData,
    ErrorType,
    Global,
    LexerActionState,
    LexerErrorType,
  >,
) -> HashSet<GrammarId> {
  // collect all grammars in grammar rules only,
  // don't collect grammar rules' NTs, because
  // some NTs might not appear in grammar rules (entry-only NTs).
  // when entry-only NTs appear in parser's buffer (e.g. end of parsing),
  // the parsing may stop. see [[@get_next_by_reduced_grammar]]
  gr_repo
    .grs()
    .iter()
    .map(|gr| gr.rule().iter().map(|g| g.id().clone()).collect::<Vec<_>>())
    .flat_map(|ids| ids.into_iter())
    .collect()
}

fn calc_first_sets<
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
>(
  nt_closures: &HashMap<
    GrammarId,
    Vec<
      Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
    >,
  >,
) -> HashMap<GrammarId, HashSet<GrammarId>> {
  let mut result = HashMap::new();

  nt_closures.iter().for_each(|(nt, grs)| {
    let grammar_set = result.entry(nt.clone()).or_insert_with(|| HashSet::new());
    // for each direct/indirect grammar rule, add first grammar to first set
    // including T and NT
    grs.iter().for_each(|gr| {
      grammar_set.insert(gr.rule()[0].id().clone());
    });
  });

  result
}

fn calc_follow_sets<
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
>(
  gr_repo: &GrammarRuleRepo<
    TKind,
    NTKind,
    ASTData,
    ErrorType,
    Global,
    LexerActionState,
    LexerErrorType,
  >,
  first_sets: &HashMap<GrammarId, HashSet<GrammarId>>,
) -> HashMap<GrammarId, HashSet<GrammarId>> {
  let mut result = HashMap::new();

  // collect follow from grammar rules
  gr_repo.grs().iter().for_each(|gr| {
    gr.rule().iter().enumerate().for_each(|(i, g)| {
      if i < gr.rule().len() - 1 {
        let next_grammar = &gr.rule()[i + 1];
        // next grammar exists, merge the current grammar's follow set with next grammar
        let grammar_set = result
          .entry(g.id().clone())
          .or_insert_with(|| HashSet::new());
        grammar_set.insert(next_grammar.id().clone());
        // if next grammar is also NT, merge with its first set
        if let GrammarKind::NT(_) = next_grammar.kind() {
          // every NT should have a first set, so we can unwrap the result
          grammar_set.extend(first_sets.get(&next_grammar.id()).unwrap().clone());
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
      let nt_follow = result
        .entry(gr.nt().id().clone())
        // some NT may not appear in grammar rules (e.g. entry NT)
        // so we have to insert new value for this situation
        .or_insert_with(|| HashSet::new())
        .clone(); // TODO: prevent the clone

      // last grammar may not already in result
      // since we only collect the result using the second last grammar
      // so we have to do a if-let check here
      if let Some(last_grammar_follow) = result.get_mut(&last_grammar.id()) {
        let len = last_grammar_follow.len();
        last_grammar_follow.extend(nt_follow);
        if last_grammar_follow.len() != len {
          changed = true;
        }
      }
    });

    if !changed {
      break;
    }
  }

  result
}
