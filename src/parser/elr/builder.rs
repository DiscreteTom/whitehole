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

pub struct ParserBuilderGrammar<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> {
  pub kind: GrammarKind<TKind, NTKind>,
  pub expect: bool,
}

impl<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> ParserBuilderGrammar<TKind, NTKind> {
  pub fn expect(mut self, value: bool) -> Self {
    self.expect = value;
    self
  }
}

#[allow(non_snake_case)]
pub fn T<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>>(
  t: TKind,
) -> ParserBuilderGrammar<TKind, NTKind> {
  ParserBuilderGrammar {
    kind: GrammarKind::T(t),
    expect: false,
  }
}
#[allow(non_snake_case)]
pub fn NT<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>>(
  nt: NTKind,
) -> ParserBuilderGrammar<TKind, NTKind> {
  ParserBuilderGrammar {
    kind: GrammarKind::NT(nt),
    expect: false,
  }
}
#[allow(non_snake_case)]
pub fn Literal<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>>(
  s: impl Into<String>,
) -> ParserBuilderGrammar<TKind, NTKind> {
  ParserBuilderGrammar {
    kind: GrammarKind::Literal(s.into()),
    expect: false,
  }
}

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
    rule: impl Into<Vec<ParserBuilderGrammar<TKind, NTKind>>>,
    traverser: Traverser<TKind, NTKind, ASTData, ErrorType, Global>,
  ) -> Self {
    let rule = rule.into();
    let expect = rule
      .iter()
      .enumerate()
      .filter_map(|(i, g)| if g.expect { Some(i) } else { None })
      .collect();
    let rule = rule
      .into_iter()
      .map(|g| self.grammars.get_or_create(g.kind).clone())
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
