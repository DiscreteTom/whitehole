use super::state::{State, StateId};
use crate::{lexer::token::TokenKind, parser::elr::grammar::grammar_rule::GrammarRule};
use std::{collections::HashMap, rc::Rc};

pub struct StateRepo<Kind: TokenKind + Clone, ASTData: 'static, ErrorType: 'static, Global: 'static>
{
  states: HashMap<StateId, Rc<State<Kind, ASTData, ErrorType, Global>>>,
}

impl<Kind: TokenKind + Clone, ASTData: 'static, ErrorType: 'static, Global: 'static>
  StateRepo<Kind, ASTData, ErrorType, Global>
{
  // TODO: only available when enable feature `generate`?
  pub fn with_entry(
    entry_candidates: Rc<Vec<Rc<GrammarRule<Kind, ASTData, ErrorType, Global>>>>,
  ) -> Self {
    let state_id = 0;
    let entry_state = Rc::new(State::new(state_id, entry_candidates, 0));
    let mut states = HashMap::new();
    states.insert(state_id, entry_state);
    Self { states }
  }

  // TODO: only available when enable feature `generate`?
  // pub fn get_next(
  //   &mut self,
  //   current: &State<Kind, ASTData, ErrorType, Global>,
  // ) -> Option<Rc<State<Kind, ASTData, ErrorType, Global>>> {
  //   None
  // }
}
