use super::{
  dfa::generate::{
    builder::build_dfa, grammar_repo::GrammarRepo, grammar_rule_repo::GrammarRuleRepo,
  },
  grammar::grammar::GrammarKind,
  parser::Parser,
};
use crate::{
  lexer::{
    token::{TokenKind, TokenKindId},
    trimmed::TrimmedLexer,
  },
  parser::traverser::Traverser,
};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

pub struct ParserBuilder<
  TKind: TokenKind<TKind>,
  NTKind: TokenKind<NTKind> + Clone,
  ASTData: 'static,
  ErrorType: 'static,
  Global: 'static,
> {
  grammars: GrammarRepo<TKind, NTKind>,
  gr_repo: GrammarRuleRepo<TKind, NTKind, ASTData, ErrorType, Global>,
}

impl<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  > Default for ParserBuilder<TKind, NTKind, ASTData, ErrorType, Global>
{
  fn default() -> Self {
    Self {
      grammars: GrammarRepo::default(),
      gr_repo: GrammarRuleRepo::default(),
    }
  }
}

impl<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  > ParserBuilder<TKind, NTKind, ASTData, ErrorType, Global>
{
  pub fn define(
    mut self,
    nt: NTKind,
    rule: Vec<(GrammarKind<TKind, NTKind>, bool)>,
    traverser: Traverser<TKind, NTKind, ASTData, ErrorType, Global>,
  ) -> Self {
    let expect = rule
      .iter()
      .enumerate()
      .filter_map(|(i, (_, e))| if *e { Some(i) } else { None })
      .collect();
    let rule = rule
      .into_iter()
      .map(|(g, _)| self.grammars.get_or_create(g).clone())
      .collect();
    self.gr_repo.push(
      self.grammars.get_or_create_nt(nt).clone(),
      rule,
      expect,
      traverser,
    );
    self
  }
  pub fn build<'buffer, LexerActionState: Clone + Default, LexerErrorType>(
    self,
    entry_nts: HashSet<TokenKindId<NTKind>>,
    lexer: impl Into<TrimmedLexer<'buffer, TKind, LexerActionState, LexerErrorType>>,
    global: Global,
  ) -> Parser<'buffer, TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>
  {
    // collect known nts
    let nts = self
      .gr_repo
      .grs()
      .iter()
      .map(|gr| gr.nt().id().clone())
      .collect();
    Parser::new(
      build_dfa(nts, entry_nts, self.gr_repo),
      lexer.into(),
      Rc::new(RefCell::new(global)),
    )
  }
}
