use crate::{
  lexer::{
    expectation::Expectation,
    token::{Range, TokenKind, TokenKindId},
    trimmed::TrimmedLexer,
  },
  parser::{
    ast::ASTNode,
    elr::{
      builder::{
        conflict::{Conflict, ConflictKind},
        reduce_context::ReduceContext,
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
    lexer: &TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    reducing_stack: &Vec<usize>,
    entry_nts: &HashSet<GrammarId>,
    follow_sets: &HashMap<GrammarId, HashSet<Rc<Grammar<TKind, NTKind>>>>,
    conflicts: &Vec<Conflict<GrammarRuleId, Rc<Grammar<TKind, NTKind>>>>,
  ) -> Option<CandidateTryReduceOutput<ASTNode<'buffer, TKind, NTKind, ASTData, ErrorType, Global>>>
  {
    if self.digested != self.gr.rule().len() {
      // this grammar rule is not fully digested, skip
      return None;
    }

    let matched = &reducing_stack[reducing_stack.len() - self.gr.rule().len()..];
    // TODO: check conflicts, etc.

    let ctx = ReduceContext::new(matched, buffer, reducing_stack, lexer);

    // do LR(1) peek, check whether the next token match current's follow set
    // first we need to make sure there is a next token
    // since the lexer is already trimmed, we only need to check if the rest is empty
    let next_token_exists = lexer.state().rest().len() > 0;
    let mut next_token = None;
    if next_token_exists {
      if entry_nts.contains(self.gr.nt().id()) {
        // TODO: feature: ignoreEntryFollow
        // entry NT, no need to check follow set if `ignoreEntryFollow` is set
        // e.g. when we parse `int a; int b;`, we don't need to check follow set for `;`
        // TODO: if the entry NT's follow set is empty, can we ignore the next check and accept it directly?
      } else {
        // not entry NT, or not ignore entry follow(treat the entry NT as normal NT)
        for grammar in follow_sets.get(self.gr.nt().id()).unwrap() {
          let expectation = match grammar.kind() {
            GrammarKind::NT(_) => continue, // skip NT
            GrammarKind::T(kind) => Expectation::from(kind),
            GrammarKind::Literal(literal) => Expectation::from(literal.as_str()),
          };
          // TODO: prevent clone lexer
          match lexer.clone().lex_expect(expectation).token {
            Some(token) => {
              // found valid follow, stop checking
              next_token = Some(token);
              break;
            }
            None => continue,
          }
        }

        if next_token.is_none() {
          // no valid follow found, reject
          return None;
        }
      }
    }

    // TODO: check conflicts

    // check rejecter
    if (self.gr.rejecter)(&ctx) {
      return None;
    }

    // accept
    // TODO: exec callback
    // TODO: return next token
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
        // TODO: set data & error
        None,
        None,
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
