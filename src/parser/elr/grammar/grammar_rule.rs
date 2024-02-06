use super::grammar::{Grammar, GrammarId, GrammarType};
use crate::{
  lexer::{expectation::Expectation, token::TokenKind, trimmed::TrimmedLexer},
  parser::ast::ASTNode,
};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

pub struct GrammarRule<Kind: TokenKind> {
  rule: Vec<Rc<Grammar<Kind>>>,
  nt: Kind,
  expect: HashSet<usize>,
}

impl<Kind: TokenKind> GrammarRule<Kind> {
  pub fn new(nt: Kind, rule: Vec<Rc<Grammar<Kind>>>, expect: HashSet<usize>) -> Self {
    Self { rule, nt, expect }
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
    lexer: &TrimmedLexer<'buffer, Kind, LexerActionState, LexerErrorType>,
    lexed_grammars: &mut HashSet<GrammarId>,
    lexed_without_expectation: bool,
    global: &Rc<RefCell<Global>>,
  ) -> Option<
    LexGrammarOutput<
      ASTNode<Kind, ASTData, ErrorType, Global>,
      TrimmedLexer<'buffer, Kind, LexerActionState, LexerErrorType>,
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
      match current.grammar_type() {
        GrammarType::NT => {
          // the current grammar is not a T, skip
          return None;
        }
        GrammarType::T => {
          // if current grammar is already lexed, skip
          if lexed_grammars.contains(&current.id()) {
            return None;
          }

          // mark this grammar as done, no matter if the lex is successful
          lexed_grammars.insert(current.id());

          let expectation = if expectational_lex {
            match current.text() {
              Some(text) => Expectation::from(current.kind()).text(text.as_str()),
              None => Expectation::from(current.kind()),
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
