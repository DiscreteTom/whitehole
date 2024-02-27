use crate::{
  lexer::{
    expectation::Expectation,
    token::{Range, Token, TokenKind, TokenKindId},
    trimmed::TrimmedLexer,
  },
  parser::{
    ast::ASTNode,
    elr::{
      builder::{
        conflict::{Conflict, ConflictKind},
        reduce_context::ReduceContext,
        resolver::ResolvedConflictConditionNext,
      },
      grammar::{
        grammar::{Grammar, GrammarId, GrammarKind},
        grammar_rule::{GrammarRule, GrammarRuleId},
      },
    },
    traverser::default_traverser,
  },
};
use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
  rc::Rc,
};

use super::utils::lex_grammar;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, PartialOrd, Ord)]
pub struct CandidateId(pub usize);

pub struct Candidate<
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
  LexerActionState: Default + Clone + 'static,
  LexerErrorType: 'static,
> {
  id: CandidateId,
  gr: Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>,
  digested: usize,
}

impl<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Default + Clone + 'static,
    LexerErrorType: 'static,
  > Candidate<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>
{
  pub fn new(
    id: CandidateId,
    gr: Rc<
      GrammarRule<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
    >,
    digested: usize,
  ) -> Self {
    Self { id, gr, digested }
  }

  pub fn id(&self) -> &CandidateId {
    &self.id
  }
  pub fn gr(
    &self,
  ) -> &Rc<GrammarRule<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>>
  {
    &self.gr
  }
  pub fn digested(&self) -> usize {
    self.digested
  }

  pub fn current(&self) -> Option<&Rc<Grammar<TKind, NTKind>>> {
    self.gr.rule().get(self.digested)
  }
  pub fn can_digest_more(&self) -> bool {
    self.digested < self.gr.rule().len() - 1
  }

  pub fn try_lex_with_expectation<'buffer>(
    &self,
    lexer: &TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    lexed_grammars: &mut HashSet<GrammarId>,
    global: &Rc<RefCell<Global>>,
  ) -> Option<
    CandidateTryLexOutput<
      ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>,
      TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    >,
  > {
    if !self.gr.expect.contains(&self.digested) {
      // current grammar doesn't require an expectational lex
      // so we can skip
      return None;
    }

    self.current().and_then(|current| {
      let (expectation, grammar_id) = match current.kind() {
        GrammarKind::NT(_) => {
          // the current grammar is an NT, not lex-able, skip
          return None;
        }
        GrammarKind::T(t) => (Expectation::from(t), current.id()),
        GrammarKind::Literal(text) => (Expectation::from(text.as_str()), current.id()),
      };

      // if current grammar is already lexed
      // the parsing state should already tried to reduce with the grammar and failed
      // and this is a re-lex, so we can skip the expectational lex
      if lexed_grammars.contains(grammar_id) {
        return None;
      }
      // else, mark this grammar as done, no matter if the lex is successful
      // because even the lex failed, we should not try to lex it again
      lexed_grammars.insert(grammar_id.clone());

      lex_grammar(expectation, lexer, global).map(|output| CandidateTryLexOutput {
        node: output.node,
        lexer: output.lexer,
        grammar_id: current.id().clone(),
      })
    })
  }

  pub fn try_accept_t_node_without_expectation<'buffer>(
    &self,
    t_kind_id: &TokenKindId<TKind>,
    text: &'buffer str,
  ) -> Option<GrammarId> {
    self.current().and_then(|current| {
      if match current.kind() {
        GrammarKind::NT(_) => {
          // the current grammar is an NT, not lex-able, skip
          false
        }
        GrammarKind::T(t) => &t.id() == t_kind_id,
        GrammarKind::Literal(literal) => literal.as_str() == text,
      } {
        Some(current.id().clone())
      } else {
        None
      }
    })
  }

  pub fn try_reduce<'buffer>(
    &self,
    buffer: &Vec<ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>>,
    next_token: &Option<Token<'buffer, TKind, LexerErrorType>>,
    lexer: &TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    reducing_stack: &Vec<usize>,
    entry_nts: &HashSet<GrammarId>,
    follow_sets: &HashMap<GrammarId, HashSet<Rc<Grammar<TKind, NTKind>>>>,
    conflicts: Option<&Vec<Conflict<GrammarRuleId, Rc<Grammar<TKind, NTKind>>>>>,
  ) -> Option<CandidateTryReduceOutput<ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>>>
  {
    if self.digested != self.gr.rule().len() {
      // this grammar rule is not fully digested, skip
      return None;
    }

    let matched = &reducing_stack[reducing_stack.len() - self.gr.rule().len()..];
    let mut ctx = ReduceContext::new(matched, buffer, reducing_stack, next_token, lexer);

    // do LR(1) peek, check whether the next token match current's follow set
    if let Some(token) = next_token {
      // check if any follow grammar match the token
      let mut mismatch = true;
      for grammar in follow_sets.get(self.gr.nt().id()).unwrap() {
        match grammar.kind() {
          GrammarKind::NT(_) => continue, // NT is not lex-able, skip
          GrammarKind::T(kind) => {
            if kind.id() == token.kind.id() {
              // found valid follow, stop checking
              mismatch = false;
              break;
            }
          }
          GrammarKind::Literal(text) => {
            if text.as_str() == token.content {
              // found valid follow, stop checking
              mismatch = false;
              break;
            }
          }
        }
      }
      if mismatch {
        // no valid follow found, reject to reduce
        return None;
      }
    } else {
      // next token is None, check if the current grammar is an entry NT
      // TODO: don't check entry NT, check end set?
      if !entry_nts.contains(self.gr.nt().id()) {
        // not an entry NT, reject to reduce
        return None;
      }
    }

    // check conflicts
    if let Some(conflicts) = conflicts {
      for c in conflicts {
        // check EOF for RR conflict
        if c.kind == ConflictKind::ReduceReduce {
          if next_token.is_none() && c.condition.eof {
            // reach EOF and this is an R-R conflict at EOF
            // try to find a resolver, we only apply the first resolver, so we use `find`
            if let Some(r) = self.gr.resolved_conflicts.iter().find(|r| {
              r.kind == ConflictKind::ReduceReduce && r.another_rule == c.another && r.condition.eof
            }) {
              if !(r.accepter)(&ctx) {
                // rejected by resolver
                return None;
              }
            } else {
              // resolver not found
              // TODO: optimize error message
              unreachable!("Every conflict should have a resolver")
            }
          }
          // else, no need to handle EOF
        }

        // check if any next grammar in the conflict match the next token
        // no matter if it's RR or SR conflict
        match next_token {
          None => {
            // no next token, skip
            continue;
          }
          Some(token) => {
            // TODO: ensure conflicts have no overlap next grammar so we can iter the grammar without duplicated calculation
            for grammar in c.condition.next.iter().filter(|g| {
              match g.kind() {
                // [[@next can only be T or Literal]]
                GrammarKind::NT(_) => unreachable!("Next can only be T or Literal"),
                GrammarKind::T(kind) => kind.id() == token.kind.id(),
                GrammarKind::Literal(text) => text.as_str() == token.content,
              }
            }) {
              // find resolver
              // TODO: can this process pre-calculated? just provide a Map<GrammarId, Accepter>?
              if let Some(r) = self.gr.resolved_conflicts.iter().find(|r| {
                r.kind == c.kind
                  && r.another_rule == c.another
                  && match &r.condition.next {
                    ResolvedConflictConditionNext::Any => true,
                    ResolvedConflictConditionNext::Some(next) => next.contains(grammar.id()),
                  }
              }) {
                if !(r.accepter)(&ctx) {
                  // rejected by resolver
                  return None;
                }
              } else {
                // TODO: optimize error message
                unreachable!("Every conflict should have a resolver")
              }
            }
          }
        }
      }
    }

    // check rejecter
    if (self.gr.rejecter)(&ctx) {
      return None;
    }

    // accept, call callback and return output
    (self.gr.callback)(&mut ctx);
    Some(CandidateTryReduceOutput {
      node: ASTNode::new_nt(
        match self.gr.nt().kind() {
          GrammarKind::NT(kind) => kind.clone(),
          _ => unreachable!(),
        },
        // TODO: is range needed?
        Range {
          start: buffer[matched[0]].range.start,
          end: buffer[matched[matched.len() - 1]].range.end,
        },
        Vec::from(matched),
        buffer[matched[0]].global.clone(),
        ctx.data,
        ctx.error,
        self
          .gr
          .traverser
          .as_ref()
          .map(|t| t.clone())
          .unwrap_or(Rc::new(default_traverser)),
      ),
      nt_grammar_id: self.gr.nt().id().clone(),
      reduced: self.gr.rule().len(),
    })
  }
}

pub struct CandidateTryLexOutput<NodeType, LexerType> {
  pub node: NodeType,
  pub lexer: LexerType,
  pub grammar_id: GrammarId,
}

pub struct CandidateTryReduceOutput<NodeType> {
  pub node: NodeType,
  pub nt_grammar_id: GrammarId,
  pub reduced: usize,
}
