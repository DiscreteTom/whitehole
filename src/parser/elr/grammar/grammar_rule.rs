use super::grammar::{Grammar, GrammarKind};
use crate::{
  lexer::{expectation::Expectation, token::TokenKind, trimmed::TrimmedLexer},
  parser::ast::TNode,
};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

pub struct GrammarRule<TKind: TokenKind, NTKind: TokenKind> {
  rule: Vec<Rc<Grammar<TKind, NTKind>>>,
  nt: NTKind,
  expect: HashSet<usize>,
}

impl<TKind: TokenKind, NTKind: TokenKind> GrammarRule<TKind, NTKind> {
  pub fn new(nt: NTKind, rule: Vec<Rc<Grammar<TKind, NTKind>>>, expect: HashSet<usize>) -> Self {
    Self { rule, nt, expect }
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

  pub fn try_lex<
    'buffer,
    ASTData: 'static,
    ErrorType: 'static,
    LexerActionState: Default + Clone,
    LexerErrorType,
    Global: 'static,
  >(
    &self,
    index: usize,
    lexer: &TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    _lexed_grammars: &mut HashSet<usize>,
    lexed_without_expectation: bool,
    global: &Rc<RefCell<Global>>,
  ) -> Option<
    LexGrammarOutput<
      TNode<TKind, NTKind, ASTData, ErrorType, Global>,
      TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    >,
  > {
    let expectational_lex = self.expect.contains(&index);

    if !expectational_lex && lexed_without_expectation {
      // current grammar doesn't require an expectational lex
      // and non-expectational lex has already been done
      // so we can skip
      return None;
    }

    self.at(index).and_then(|current| {
      match current.kind() {
        GrammarKind::NT(..) => {
          // the current grammar is not a T, skip
          return None;
        }
        GrammarKind::T(kind) => {
          // if current grammar is already lexed, skip
          // TODO: define grammar id
          // if lexed_grammars.contains(&current.id()) {
          //   return None;
          // }

          // mark this grammar as done, no matter if the lex is successful
          // TODO: define grammar id
          // lexed_grammars.insert(current.id());

          let expectation = if expectational_lex {
            match current.text() {
              Some(text) => Expectation::from(kind).text(text.as_str()),
              None => Expectation::from(kind),
            }
          } else {
            // no expectation
            Expectation::default()
          };

          Self::lex_grammar::<ASTData, ErrorType, _, _, _>(expectation, lexer, global)
        }
      }
    })
  }

  fn lex_grammar<
    'buffer,
    ASTData,
    ErrorType,
    LexerActionState: Default + Clone,
    LexerErrorType,
    Global,
  >(
    expectation: Expectation<TKind>,
    lexer: &TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>,
    global: &Rc<RefCell<Global>>,
  ) -> Option<
    LexGrammarOutput<
      TNode<TKind, NTKind, ASTData, ErrorType, Global>,
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
      let node = TNode::new(token.kind, token.range, None, None, None, global.clone());
      LexGrammarOutput { node, lexer }
    })
  }
}

pub struct LexGrammarOutput<NodeType, LexerType> {
  pub node: NodeType,
  pub lexer: LexerType,
}
