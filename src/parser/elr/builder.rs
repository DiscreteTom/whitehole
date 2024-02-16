pub mod conflict;
pub mod grammar_rule_context_builder;
pub mod reduce_context;
pub mod temp_grammar_rule;

use self::{
  grammar_rule_context_builder::GrammarRuleContextBuilder, temp_grammar_rule::TempGrammarRule,
};
use super::{
  dfa::generate::{
    builder::build_dfa, grammar_repo::GrammarRepo, grammar_rule_repo::GrammarRuleRepo,
  },
  grammar::{grammar::GrammarKind, grammar_rule::GrammarRuleId},
  parser::Parser,
};
use crate::lexer::{stateless::StatelessLexer, token::TokenKind};
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
  TKind: TokenKind<TKind> + 'static,
  NTKind: TokenKind<NTKind> + Clone + 'static,
  ASTData: 'static = (),
  ErrorType: 'static = (),
  Global: 'static = (),
  LexerActionState: Clone + Default + 'static = (),
  LexerErrorType: 'static = (),
> {
  lexer: StatelessLexer<TKind, LexerActionState, LexerErrorType>,
  entry_nts: Vec<NTKind>,
  global: Global,
  grammars: GrammarRepo<TKind, NTKind>,
  gr_repo:
    GrammarRuleRepo<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>,
  defined_grs: HashSet<GrammarRuleId>,
}

impl<
    TKind: TokenKind<TKind> + Clone,
    NTKind: TokenKind<NTKind> + Clone + 'static,
    ASTData: 'static,
    ErrorType: 'static,
    Global: 'static,
    LexerActionState: Clone + Default + 'static,
    LexerErrorType: 'static,
  > ParserBuilder<TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>
{
  pub fn new(
    lexer: impl Into<StatelessLexer<TKind, LexerActionState, LexerErrorType>>,
    entry_nts: impl Into<Vec<NTKind>>,
    global: Global,
  ) -> Self {
    Self {
      lexer: lexer.into(),
      entry_nts: entry_nts.into(),
      global,
      grammars: GrammarRepo::default(),
      gr_repo: GrammarRuleRepo::default(),
      defined_grs: HashSet::new(),
    }
  }

  pub fn define(self, gr: Rc<TempGrammarRule<TKind, NTKind>>) -> Self {
    self.define_with(gr, |_| {})
  }

  pub fn define_with<'a, 'buffer: 'a, F>(
    mut self,
    gr: Rc<TempGrammarRule<TKind, NTKind>>,
    f: F,
  ) -> Self
  where
    F: FnOnce(
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
    let expect = gr
      .rule()
      .iter()
      .enumerate()
      .filter_map(|(i, g)| if g.expect { Some(i) } else { None })
      .collect();
    let rule = gr
      .rule()
      .iter()
      .map(|g| self.grammars.get_or_create(g.kind.clone()).clone())
      .collect();

    let mut ctx = GrammarRuleContextBuilder::default();
    f(&mut ctx);

    let gr = self.gr_repo.get_or_add(
      self.grammars.get_or_create_nt(gr.nt().clone()).clone(),
      rule,
      expect,
      ctx.rejecter,
      None,
    );

    // ensure we don't define the same grammar rule twice
    if !self.defined_grs.insert(gr.id().clone()) {
      panic!("Grammar rule already defined: {:?}", gr.id());
    }

    self
  }

  pub fn build<'buffer>(
    self,
    input: &'buffer str,
  ) -> Parser<'buffer, TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>
  {
    Parser::new(
      build_dfa(
        self.entry_nts.into_iter().map(|e| e.id()).collect(),
        self.gr_repo,
      ),
      self.lexer.into_lexer(input).into(),
      Rc::new(RefCell::new(self.global)),
    )
  }
}
