pub mod conflict;
pub mod grammar_rule_context_builder;
pub mod reduce_context;
pub mod resolver;
pub mod temp_grammar_rule;
pub mod temp_resolver;

use self::{
  grammar_rule_context_builder::GrammarRuleContextBuilder, resolver::ResolvedConflict,
  temp_grammar_rule::TempGrammarRule,
};
use super::{
  dfa::generate::{
    builder::build_dfa, grammar_repo::GrammarRepo, grammar_rule_repo::GrammarRuleRepo,
  },
  grammar::{
    grammar::{GrammarId, GrammarKind},
    grammar_rule::GrammarRuleId,
  },
  parser::Parser,
};
use crate::lexer::{stateless::StatelessLexer, token::TokenKind};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

pub struct ParserBuilderGrammar<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> {
  pub kind: GrammarKind<TKind, NTKind>,
  /// Only effective for T/Literal.
  pub expected: bool,
}

impl<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> ParserBuilderGrammar<TKind, NTKind> {
  /// Only effective for T/Literal.
  pub fn expect(mut self, value: bool) -> Self {
    self.expected = value;
    self
  }
}

#[allow(non_snake_case)]
pub fn T<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>>(
  t: TKind,
) -> ParserBuilderGrammar<TKind, NTKind> {
  ParserBuilderGrammar {
    kind: GrammarKind::T(t),
    expected: false,
  }
}
#[allow(non_snake_case)]
pub fn NT<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>>(
  nt: NTKind,
) -> ParserBuilderGrammar<TKind, NTKind> {
  ParserBuilderGrammar {
    kind: GrammarKind::NT(nt),
    expected: false,
  }
}
#[allow(non_snake_case)]
pub fn Literal<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>>(
  s: impl Into<String>,
) -> ParserBuilderGrammar<TKind, NTKind> {
  ParserBuilderGrammar {
    kind: GrammarKind::Literal(s.into()),
    expected: false,
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
  entry_nts: HashSet<GrammarId>,
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
    let mut grammars = GrammarRepo::default();
    let entry_nts = entry_nts
      .into()
      .into_iter()
      .map(|nt| grammars.get_or_create_nt(nt).id().clone())
      .collect();
    Self {
      lexer: lexer.into(),
      entry_nts,
      global,
      grammars,
      gr_repo: GrammarRuleRepo::default(),
      defined_grs: HashSet::new(),
    }
  }

  pub fn define(self, gr: Rc<TempGrammarRule<TKind, NTKind>>) -> Self {
    self.define_with(gr, |ctx| ctx)
  }

  pub fn define_with<'a, 'buffer: 'a, F>(
    mut self,
    gr: Rc<TempGrammarRule<TKind, NTKind>>,
    f: F,
  ) -> Self
  where
    F: FnOnce(
      GrammarRuleContextBuilder<
        TKind,
        NTKind,
        ASTData,
        ErrorType,
        Global,
        LexerActionState,
        LexerErrorType,
      >,
    ) -> GrammarRuleContextBuilder<
      TKind,
      NTKind,
      ASTData,
      ErrorType,
      Global,
      LexerActionState,
      LexerErrorType,
    >,
  {
    let expect = gr
      .rule()
      .iter()
      .enumerate()
      .filter_map(|(i, g)| if g.expected { Some(i) } else { None })
      .collect();
    let rule = gr
      .rule()
      .iter()
      .map(|g| self.grammars.get_or_create(g.kind.clone()).clone())
      .collect();

    let ctx = f(GrammarRuleContextBuilder::default());

    // prepare resolved conflicts
    let resolved_conflicts = ctx
      .resolved_conflicts
      .into_iter()
      .map(|r| ResolvedConflict {
        kind: r.kind,
        another_rule: self
          .gr_repo
          .get_or_add(
            self
              .grammars
              .get_or_create_nt(r.another_rule.nt().clone())
              .clone(),
            r.another_rule
              .rule()
              .iter()
              .map(|g| self.grammars.get_or_create(g.kind.clone()).clone())
              .collect(),
          )
          .id()
          .clone(),
        accepter: r.accepter,
        condition: r
          .condition
          .into_resolved_conflict_condition(&mut self.grammars),
      })
      .collect();

    // the new grammar rule
    let gr = self.gr_repo.get_or_add(
      self.grammars.get_or_create_nt(gr.nt().clone()).clone(),
      rule,
    );

    // ensure we don't define the same grammar rule twice
    if !self.defined_grs.insert(gr.id().clone()) {
      panic!("Grammar rule already defined: {:?}", gr.id());
    }

    // since this is a define action, overwrite properties
    gr.expect = expect;
    gr.rejecter = ctx.rejecter;
    gr.resolved_conflicts = resolved_conflicts;

    self
  }

  pub fn build<'buffer>(
    self,
    input: &'buffer str,
  ) -> Parser<'buffer, TKind, NTKind, ASTData, ErrorType, Global, LexerActionState, LexerErrorType>
  {
    // TODO: check if all grammar rules are defined
    // because maybe some grammar rules only appears in resolvers but never defined
    Parser::new(
      build_dfa(
        self.entry_nts,
        self.gr_repo.grs.into_iter().map(|gr| Rc::new(gr)).collect(),
      ),
      self.lexer.into_lexer(input).into(),
      Rc::new(RefCell::new(self.global)),
    )
  }
}
