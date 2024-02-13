use super::{
  dfa::generate::{
    builder::build_dfa, grammar_repo::GrammarRepo, grammar_rule_repo::GrammarRuleRepo,
  },
  grammar::grammar::GrammarKind,
  parser::Parser,
};
use crate::{
  lexer::{stateless::StatelessLexer, token::TokenKind},
  parser::traverser::Traverser,
};
use std::{cell::RefCell, rc::Rc};

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
  ASTData: 'static = (),
  ErrorType: 'static = (),
  Global: 'static = (),
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
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
  > ParserBuilder<TKind, NTKind, ASTData, ErrorType, Global>
{
  pub fn define(
    mut self,
    nt: NTKind,
    rule: impl Into<Vec<ParserBuilderGrammar<TKind, NTKind>>>,
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
    self.gr_repo.get_or_add(
      self.grammars.get_or_create_nt(nt).clone(),
      rule,
      expect,
      None,
    );
    self
  }

  pub fn build<'buffer, LexerActionState: Clone + Default, LexerErrorType>(
    self,
    entry_nts: impl Into<Vec<NTKind>>,
    lexer: impl Into<StatelessLexer<TKind, LexerActionState, LexerErrorType>>,
    global: Global,
    input: &'buffer str,
  ) -> Parser<'buffer, TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>
  {
    let entry_nts = entry_nts.into().into_iter().map(|e| e.id()).collect();
    // collect known nts
    let nts = self
      .gr_repo
      .grs()
      .iter()
      .map(|gr| gr.nt().id().clone())
      .collect();
    Parser::new(
      build_dfa(nts, entry_nts, self.gr_repo),
      lexer.into().into_lexer(input).into(),
      Rc::new(RefCell::new(global)),
    )
  }
}
