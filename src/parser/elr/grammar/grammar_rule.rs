use super::grammar::{Grammar, GrammarId, GrammarType};
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

pub struct GrammarRule<
  Kind: TokenKind + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  rule: Vec<Rc<Grammar<Kind>>>,
  nt: Kind,
  expect: HashSet<usize>,
  traverser: Traverser<Kind, ASTData, ErrorType, Global>,
}

impl<Kind: TokenKind + Clone, ASTData: 'static, ErrorType: 'static, Global: 'static>
  GrammarRule<Kind, ASTData, ErrorType, Global>
{
  pub fn new(
    nt: Kind,
    rule: Vec<Rc<Grammar<Kind>>>,
    expect: HashSet<usize>,
    traverser: Traverser<Kind, ASTData, ErrorType, Global>,
  ) -> Self {
    Self {
      rule,
      nt,
      expect,
      traverser,
    }
  }
  pub fn nt(&self) -> &Kind {
    &self.nt
  }
  pub fn rule(&self) -> &[Rc<Grammar<Kind>>] {
    &self.rule
  }

  pub fn at(&self, index: usize) -> Option<&Rc<Grammar<Kind>>> {
    self.rule.get(index)
  }

  pub fn try_lex<'buffer, LexerActionState: Default + Clone, LexerErrorType>(
    &self,
    digested: usize,
    lexer: &TrimmedLexer<'buffer, Kind, LexerActionState, LexerErrorType>,
    lexed_grammars: &mut HashSet<GrammarId>,
    lexed_without_expectation: &mut bool,
    global: &Rc<RefCell<Global>>,
  ) -> Option<
    LexGrammarOutput<
      ASTNode<Kind, ASTData, ErrorType, Global>,
      TrimmedLexer<'buffer, Kind, LexerActionState, LexerErrorType>,
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
      match current.grammar_type() {
        GrammarType::NT => {
          // the current grammar is not a T, skip
          return None;
        }
        GrammarType::T => {
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
              Some(text) => Expectation::from(current.kind()).text(text.as_str()),
              None => Expectation::from(current.kind()),
            }
          } else {
            // no expectation
            Expectation::default()
          };

          Self::lex_grammar(expectation, lexer, global)
        }
      }
    })
  }

  pub fn try_reduce<'buffer, LexerActionState: Default + Clone, LexerErrorType>(
    &self,
    digested: usize,
    buffer: &Vec<ASTNode<Kind, ASTData, ErrorType, Global>>,
    lexer: &TrimmedLexer<'buffer, Kind, LexerActionState, LexerErrorType>,
    reducing_stack: &Vec<usize>,
    entry_nts: &HashSet<TokenKindId>,
    follow_sets: &HashMap<TokenKindId, TokenKindId>,
  ) -> Option<ASTNode<Kind, ASTData, ErrorType, Global>> {
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
    expectation: Expectation<Kind>,
    lexer: &TrimmedLexer<'buffer, Kind, LexerActionState, LexerErrorType>,
    global: &Rc<RefCell<Global>>,
  ) -> Option<
    LexGrammarOutput<
      ASTNode<Kind, ASTData, ErrorType, Global>,
      TrimmedLexer<'buffer, Kind, LexerActionState, LexerErrorType>,
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

pub struct LexGrammarOutput<NodeType, LexerType> {
  pub node: NodeType,
  pub lexer: LexerType,
}
