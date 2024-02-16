pub mod conflict;
pub mod grammar_rule_context_builder;
pub mod reduce_context;

use self::grammar_rule_context_builder::GrammarRuleContextBuilder;
use super::{
  dfa::generate::{
    builder::build_dfa, grammar_repo::GrammarRepo, grammar_rule_repo::GrammarRuleRepo,
  },
  grammar::grammar::GrammarKind,
  parser::Parser,
};
use crate::lexer::{stateless::StatelessLexer, token::TokenKind};
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
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static = (),
  ErrorType: 'static = (),
  Global: 'static = (),
  LexerActionState: Clone + Default + 'static = (),
  LexerErrorType: 'static = (),
> {
  lexer: StatelessLexer<TKind, LexerActionState, LexerErrorType>,
  grammars: GrammarRepo<TKind, NTKind>,
  gr_repo:
    GrammarRuleRepo<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
}

impl<
    TKind: TokenKind<TKind>,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Clone + Default + 'static,
    LexerErrorType: 'static,
  > ParserBuilder<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>
{
  pub fn new(lexer: impl Into<StatelessLexer<TKind, LexerActionState, LexerErrorType>>) -> Self {
    Self {
      lexer: lexer.into(),
      grammars: GrammarRepo::default(),
      gr_repo: GrammarRuleRepo::default(),
    }
  }

  pub fn define(
    self,
    nt: NTKind,
    rule: impl Into<Vec<ParserBuilderGrammar<TKind, NTKind>>>,
  ) -> Self {
    self.define_with(nt, rule, |_| {})
  }

  pub fn define_with<'a, 'buffer: 'a, F>(
    mut self,
    nt: NTKind,
    rule: impl Into<Vec<ParserBuilderGrammar<TKind, NTKind>>>,
    f: F,
  ) -> Self
  where
    F: Fn(
      &mut GrammarRuleContextBuilder<
        TKind,
        NTKind,
        ASTData,
        ErrorType,
        Global,
        LexerActionState,
        LexerErrorType,
      >,
    ),
  {
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

    let mut ctx = GrammarRuleContextBuilder::default();
    f(&mut ctx);

    // TODO: panic if defining duplicated grammar rule
    self.gr_repo.get_or_add(
      self.grammars.get_or_create_nt(nt).clone(),
      rule,
      expect,
      ctx.rejecter,
      None,
    );
    self
  }

  pub fn build<'buffer>(
    self,
    // TODO: move entry_nts and global to constructor?
    entry_nts: impl Into<Vec<NTKind>>,
    global: Global,
    input: &'buffer str,
  ) -> Parser<'buffer, TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>
  {
    Parser::new(
      build_dfa(
        entry_nts.into().into_iter().map(|e| e.id()).collect(),
        self.gr_repo,
      ),
      self.lexer.into_lexer(input).into(),
      Rc::new(RefCell::new(global)),
    )
  }
}
