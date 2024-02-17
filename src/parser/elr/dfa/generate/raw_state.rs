use crate::{
  lexer::token::TokenKind,
  parser::elr::{
    builder::conflict::{Conflict, ConflictCondition, ConflictKind},
    dfa::{
      candidate::{Candidate, CandidateId},
      state::{State, StateId},
    },
    grammar::grammar::{Grammar, GrammarId, GrammarKind},
  },
};
use std::{
  collections::{BTreeSet, HashMap, HashSet},
  rc::Rc,
};

pub struct RawState {
  id: StateId,
  // store candidates as Rc because we also need this in [[@StateRepo.cache]].
  candidates: Rc<BTreeSet<CandidateId>>,
  next_map: HashMap<GrammarId, Option<StateId>>,
}

impl RawState {
  pub fn new(id: StateId, candidates: Rc<BTreeSet<CandidateId>>) -> Self {
    RawState {
      id,
      candidates,
      next_map: HashMap::default(),
    }
  }

  pub fn id(&self) -> &StateId {
    &self.id
  }
  pub fn candidates(&self) -> &Rc<BTreeSet<CandidateId>> {
    &self.candidates
  }

  pub fn append_next(&mut self, input_grammar_id: GrammarId, next: Option<StateId>) {
    self.next_map.insert(input_grammar_id, next);
  }

  pub fn into_state<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Default + Clone + 'static,
    LexerErrorType: 'static,
  >(
    self,
    candidates: &Vec<
      Rc<Candidate<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
    >,
    first_sets: &HashMap<GrammarId, HashSet<Rc<Grammar<TKind, NTKind>>>>,
    follow_sets: &HashMap<GrammarId, HashSet<Rc<Grammar<TKind, NTKind>>>>,
    end_set: &HashSet<GrammarId>,
  ) -> State<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType> {
    let candidates = self
      .candidates
      .iter()
      .map(|id| candidates[id.0].clone())
      .collect();

    let conflict_map = Self::calc_conflict_map(&candidates, first_sets, follow_sets, end_set);
    State::new(self.id, candidates, self.next_map, conflict_map)
  }

  fn calc_conflict_map<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Default + Clone + 'static,
    LexerErrorType: 'static,
  >(
    candidates: &Vec<
      Rc<Candidate<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
    >,
    first_sets: &HashMap<GrammarId, HashSet<Rc<Grammar<TKind, NTKind>>>>,
    follow_sets: &HashMap<GrammarId, HashSet<Rc<Grammar<TKind, NTKind>>>>,
    end_set: &HashSet<GrammarId>,
  ) -> HashMap<CandidateId, Vec<Conflict<CandidateId>>> {
    let mut res = HashMap::new();

    // first, check if there is any reduce-able candidate in the current state
    let reducer_candidates: Vec<_> = candidates.iter().filter(|c| !c.can_digest_more()).collect();

    // all reduce-able candidates may have RR conflicts with each other
    for (i, reducer) in reducer_candidates.iter().enumerate() {
      for (j, another) in reducer_candidates.iter().enumerate() {
        // prevent duplicate check & self check
        if i >= j {
          continue;
        }

        // we don't need to check if RR conflicts already exists between 2 grammar rules
        // because the collected conflicts are in the state level, not in the grammar rule level
        // and every candidate is unique

        // if there is no overlap between reducer's follow and another's follow
        // then there is no RR conflict for next, but maybe still has RR conflict when handle end of input
        let follow_overlap = follow_sets
          .get(reducer.gr().nt().id())
          .unwrap()
          .intersection(follow_sets.get(another.gr().nt().id()).unwrap())
          .map(|g| g.id().clone())
          .collect::<HashSet<_>>();

        // if reducer's NT and another's NT are both in end set, then we need to handle end of input
        let need_handle_end =
          end_set.contains(reducer.gr().nt().id()) && end_set.contains(another.gr().nt().id());

        // append conflict if needed
        if follow_overlap.len() > 0 || need_handle_end {
          res
            .entry(reducer.id().clone())
            .or_insert_with(|| Vec::new())
            .push(Conflict {
              kind: ConflictKind::ReduceReduce,
              another: another.id().clone(),
              condition: ConflictCondition {
                next: follow_overlap,
                eof: need_handle_end,
              },
            });
        }
      }
    }

    // all reduce-able candidates may have RS conflicts with non-reduce-able candidates (shifters)
    let shifter_candidates: Vec<_> = candidates
      .iter()
      // if digested === 0, this candidate has indirect RS conflict with reducer, skip it.
      // we only want direct RS conflict
      .filter(|c| c.can_digest_more() && c.digested() != 0)
      .collect();

    for reducer in &reducer_candidates {
      for shifter in &shifter_candidates {
        let shifter_current = shifter.current().unwrap();
        match shifter_current.kind() {
          GrammarKind::T(_) | GrammarKind::Literal(_) => {
            // if shifter's current is a T/Literal and is not in reducer's NT's follow
            // then there is no RS conflict
            if !follow_sets
              .get(reducer.gr().nt().id())
              .unwrap()
              .contains(shifter_current)
            {
              // no overlap, no RS conflict
            } else {
              // overlap, RS conflict

              // we don't need to check if RS conflicts already exists between 2 grammar rules
              // because the collected conflicts are in the state level, not in the grammar rule level
              // and every candidate is unique

              res
                .entry(reducer.id().clone())
                .or_insert_with(|| Vec::new())
                .push(Conflict {
                  kind: ConflictKind::ReduceShift,
                  another: shifter.id().clone(),
                  condition: ConflictCondition {
                    next: HashSet::from([shifter_current.id().clone()]),
                    eof: false, // we don't need to handle eof for RS conflict
                  },
                });
            }
          }
          GrammarKind::NT(_) => {
            // shifter's current is an NT, check if reducer's NT's follow has overlap with shifter's current's first
            let overlap = follow_sets
              .get(reducer.gr().nt().id())
              .unwrap()
              .intersection(first_sets.get(shifter_current.id()).unwrap())
              .map(|g| g.id().clone())
              .collect::<HashSet<_>>();

            if overlap.len() > 0 {
              // overlap, RS conflict

              // we don't need to check if RS conflicts already exists between 2 grammar rules
              // because the collected conflicts are in the state level, not in the grammar rule level
              // and every candidate is unique

              res
                .entry(reducer.id().clone())
                .or_insert_with(|| Vec::new())
                .push(Conflict {
                  kind: ConflictKind::ReduceShift,
                  another: shifter.id().clone(),
                  condition: ConflictCondition {
                    next: overlap,
                    eof: false, // we don't need to handle eof for RS conflict
                  },
                });
            }
          }
        }
      }
    }

    res
  }
}
