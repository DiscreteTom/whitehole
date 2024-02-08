use super::grammar::{Grammar, GrammarId, GrammarKind};
use crate::{
  lexer::{
    expectation::Expectation,
    token::{Range, TokenKind, TokenKindId},
    trimmed::TrimmedLexer,
  },
  parser::{ast::ASTNode, traverser::Traverser},
};
use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
  rc::Rc,
};

pub type GrammarRuleId = usize;

pub struct GrammarRule<
  TKind: TokenKind,
  NTKind: TokenKind,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  id: GrammarRuleId,
  rule: Vec<Rc<Grammar<TKind, NTKind>>>,
  nt: NTKind,
  expect: HashSet<usize>,
  traverser: Traverser<TKind, NTKind, ASTData, ErrorType, Global>,
}

impl<
    TKind: TokenKind,
    NTKind: TokenKind + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  > GrammarRule<TKind, NTKind, ASTData, ErrorType, Global>
{
  pub fn new(
    id: GrammarRuleId,
    nt: NTKind,
    rule: Vec<Rc<Grammar<TKind, NTKind>>>,
    expect: HashSet<usize>,
    traverser: Traverser<TKind, NTKind, ASTData, ErrorType, Global>,
  ) -> Self {
    Self {
      id,
      rule,
      nt,
      expect,
      traverser,
    }
  }
  pub fn id(&self) -> GrammarRuleId {
    self.id
  }
  pub fn nt(&self) -> &NTKind {
    &self.nt
  }
  pub fn rule(&self) -> &[Rc<Grammar<TKind, NTKind>>] {
    &self.rule
  }

  pub fn at(&self, index: usize) -> Option<&Rc<Grammar<TKind, NTKind>>> {
    self.rule.get(index)
  }

  pub fn try_lex<'buffer, LexerActionState: Default + Clone, LexerErrorType>(
    &self,
    digested: usize,
    lexer: &TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    lexed_grammars: &mut HashSet<GrammarId>,
    lexed_without_expectation: &mut bool,
    global: &Rc<RefCell<Global>>,
  ) -> Option<
    GrammarRuleTryLexOutput<
      ASTNode<TKind, NTKind, ASTData, ErrorType, Global>,
      TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    >,
  > {
    let expectational_lex = self.expect.contains(&digested);

    if !expectational_lex && *lexed_without_expectation {
      // current grammar doesn't require an expectational lex
      // and non-expectational lex has already been done
      // so we can skip
      return None;
    }

    self.at(digested).and_then(|current| {
      match current.kind() {
        GrammarKind::NT(_) => {
          // the current grammar is not a T, skip
          return None;
        }
        GrammarKind::T(kind) => {
          if expectational_lex {
            // if current grammar is already lexed, skip
            if lexed_grammars.contains(&current.id()) {
              return None;
            }
            // else, mark this grammar as done, no matter if the lex is successful
            lexed_grammars.insert(current.id());
          } else {
            // mark non-expectational lex as done, no matter if the lex is successful
            *lexed_without_expectation = true;
          }

          let expectation = if expectational_lex {
            match current.text() {
              Some(text) => Expectation::from(kind).text(text.as_str()),
              None => Expectation::from(kind),
            }
          } else {
            // no expectation
            Expectation::default()
          };

          Self::lex_grammar(expectation, lexer, global).map(|output| GrammarRuleTryLexOutput {
            node: output.node,
            lexer: output.lexer,
            grammar_id: current.id(),
          })
        }
      }
    })
  }

  pub fn try_reduce<'buffer, LexerActionState: Default + Clone, LexerErrorType>(
    &self,
    digested: usize,
    buffer: &Vec<ASTNode<TKind, NTKind, ASTData, ErrorType, Global>>,
    lexer: &TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    reducing_stack: &Vec<usize>,
    entry_nts: &HashSet<TokenKindId>,
    follow_sets: &HashMap<TokenKindId, TokenKindId>,
  ) -> Option<ASTNode<TKind, NTKind, ASTData, ErrorType, Global>> {
    if digested != self.rule.len() - 1 {
      // this grammar rule is not fully digested, skip
      return None;
    }

    let matched = &reducing_stack[reducing_stack.len() - self.rule.len()..];
    // TODO: set name
    // TODO: check conflicts, rejecter, etc.

    // accept
    Some(ASTNode::new_nt(
      self.nt.clone(),
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
      self.traverser.clone(),
    ))
  }

  fn lex_grammar<'buffer, LexerActionState: Default + Clone, LexerErrorType>(
    expectation: Expectation<TKind>,
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
    res.token.map(move |token| {
      let lexer = res.lexer.into_trimmed().trimmed_lexer;
      // TODO: set node data
      let node = ASTNode::new_t(token.kind, token.range, global.clone(), None, None);
      LexGrammarOutput { node, lexer }
    })
  }
}

struct LexGrammarOutput<NodeType, LexerType> {
  pub node: NodeType,
  pub lexer: LexerType,
}

pub struct GrammarRuleTryLexOutput<NodeType, LexerType> {
  pub node: NodeType,
  pub lexer: LexerType,
  pub grammar_id: GrammarId,
}
