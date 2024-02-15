use crate::{
  lexer::{
    expectation::Expectation,
    token::{Range, TokenKind, TokenKindId},
    trimmed::TrimmedLexer,
  },
  parser::{
    ast::ASTNode,
    elr::{
      builder::reduce_context::ReduceContext,
      grammar::{
        grammar::{Grammar, GrammarId, GrammarKind},
        grammar_rule::GrammarRule,
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

  pub fn try_lex<'buffer>(
    &self,
    lexer: &TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    lexed_grammars: &mut HashSet<GrammarId>,
    lexed_without_expectation: &mut bool,
    global: &Rc<RefCell<Global>>,
  ) -> Option<
    CandidateTryLexOutput<
      ASTNode<TKind, NTKind, ASTData, ErrorType, Global>,
      TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    >,
  > {
    let expectational_lex = self.gr.expect().contains(&self.digested);

    if !expectational_lex && *lexed_without_expectation {
      // current grammar doesn't require an expectational lex
      // and non-expectational lex has already been done
      // so we can skip
      return None;
    }

    self.current().and_then(|current| {
      let (expectation, validator) = match current.kind() {
        GrammarKind::NT(_) => {
          // the current grammar is not a T, skip
          return None;
        }
        GrammarKind::T(kind) => {
          // TODO: optimize code, reduce duplicated code
          if expectational_lex {
            // if current grammar is already lexed, skip
            // TODO: should this skip?
            if lexed_grammars.contains(&current.id()) {
              return None;
            }
            // else, mark this grammar as done, no matter if the lex is successful
            lexed_grammars.insert(current.id().clone());
          } else {
            // mark non-expectational lex as done, no matter if the lex is successful
            *lexed_without_expectation = true;
          }

          if expectational_lex {
            (
              Expectation::from(kind),
              // with expectational lex, we don't need to validate the output
              None,
            )
          } else {
            (
              // no expectation
              Expectation::default(),
              // validate the output
              Some(LexGrammarTokenValidator::TKind(kind.id())),
            )
          }
        }
        GrammarKind::Literal(text) => {
          if expectational_lex {
            // if current grammar is already lexed, skip
            if lexed_grammars.contains(&current.id()) {
              return None;
            }
            // else, mark this grammar as done, no matter if the lex is successful
            lexed_grammars.insert(current.id().clone());
          } else {
            // mark non-expectational lex as done, no matter if the lex is successful
            *lexed_without_expectation = true;
          }

          if expectational_lex {
            (
              Expectation::from(text.as_str()),
              // with expectational lex, we don't need to validate the output
              None,
            )
          } else {
            (
              // no expectation
              Expectation::default(),
              // validate the output
              Some(LexGrammarTokenValidator::Text(text)),
            )
          }
        }
      };
      Self::lex_grammar(expectation, validator, lexer, global).map(|output| CandidateTryLexOutput {
        node: output.node,
        lexer: output.lexer,
        grammar_id: current.id().clone(),
      })
    })
  }

  pub fn try_reduce<'buffer>(
    &self,
    buffer: &Vec<ASTNode<TKind, NTKind, ASTData, ErrorType, Global>>,
    lexer: &TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    reducing_stack: &Vec<usize>,
    entry_nts: &HashSet<TokenKindId<NTKind>>,
    follow_sets: &HashMap<GrammarId, HashSet<GrammarId>>,
  ) -> Option<CandidateTryReduceOutput<ASTNode<TKind, NTKind, ASTData, ErrorType, Global>>> {
    if self.digested != self.gr.rule().len() {
      // this grammar rule is not fully digested, skip
      return None;
    }

    let matched = &reducing_stack[reducing_stack.len() - self.gr.rule().len()..];
    // TODO: set name
    // TODO: check conflicts, etc.

    let ctx = ReduceContext::new(matched, buffer, reducing_stack, lexer);

    // check rejecter
    if (self.gr.rejecter())(&ctx) {
      return None;
    }

    // accept
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
          .traverser()
          .as_ref()
          .map(|t| t.clone())
          .unwrap_or(Rc::new(default_traverser)),
      ),
      nt_grammar_id: self.gr.nt().id().clone(),
      reduced: self.gr.rule().len(),
    })
  }

  fn lex_grammar<'buffer>(
    expectation: Expectation<TKind>,
    validator: Option<LexGrammarTokenValidator<TKind>>,
    lexer: &TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    global: &Rc<RefCell<Global>>,
  ) -> Option<
    LexGrammarOutput<
      ASTNode<TKind, NTKind, ASTData, ErrorType, Global>,
      TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    >,
  > {
    // because of re-lex, we may store many lexers
    // so we clone the lexer to prevent side effect.
    // we must clone the lexer here to prevent unnecessary clone.
    // you may think using peek is more efficient, but it's not,
    // since we still need to clone and store the new lexer state and action state
    // so it's actually the same.
    // TODO: don't clone the lexer if we disable re-lex or when re-lex won't happen
    let lexer = lexer.clone();

    let res = lexer.lex_expect(expectation);
    res.token.and_then(move |token| {
      // validate the output
      if let Some(v) = validator {
        match v {
          LexGrammarTokenValidator::TKind(kind_id) => {
            if token.kind.id() != kind_id {
              return None;
            }
          }
          LexGrammarTokenValidator::Text(text) => {
            if token.content() != text {
              return None;
            }
          }
        }
      }

      let lexer = res.lexer.into_trimmed().trimmed_lexer;
      // TODO: set node data
      let node = ASTNode::new_t(token.kind, token.range, global.clone(), None, None);
      Some(LexGrammarOutput { node, lexer })
    })
  }
}

enum LexGrammarTokenValidator<'a, TKind> {
  TKind(TokenKindId<TKind>),
  Text(&'a String),
}

struct LexGrammarOutput<NodeType, LexerType> {
  pub node: NodeType,
  pub lexer: LexerType,
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
