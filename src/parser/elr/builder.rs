pub mod conflict;
pub mod grammar_rule_context_builder;
pub mod lexer_panic_handler;
pub mod reduce_context;
pub mod resolver;
pub mod temp_resolver;

use self::{
  conflict::ConflictKind,
  grammar_rule_context_builder::GrammarRuleContextBuilder,
  lexer_panic_handler::{default_lexer_panic_handler, LexerPanicHandler},
  resolver::ResolvedConflict,
  temp_resolver::{ReduceReduceResolverOptions, ReduceShiftResolverOptions},
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
use crate::lexer::{stateless::StatelessLexer, token::TokenKind, trimmed::TrimmedLexer};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

pub struct ParserBuilderGrammar<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> {
  pub kind: GrammarKind<TKind, NTKind>,
  /// Only effective for T/Literal.
  pub expected: bool,
}

impl<TKind: TokenKind<TKind>, NTKind: TokenKind<NTKind>> ParserBuilderGrammar<TKind, NTKind> {
  /// Only effective for T/Literal.
  pub fn expect(mut self, value: bool) -> Self {
    if let GrammarKind::T(_) | GrammarKind::Literal(_) = self.kind {
      self.expected = value;
    } else {
      panic!("Only T and Literal can have expected value");
    }
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
  lexer_panic_handler: LexerPanicHandler<TKind, LexerActionState, LexerErrorType>,
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
      lexer_panic_handler: Box::new(default_lexer_panic_handler),
    }
  }

  pub fn define(
    &mut self,
    nt: NTKind,
    rule: impl Into<Vec<ParserBuilderGrammar<TKind, NTKind>>>,
  ) -> GrammarRuleId {
    self.define_with(nt, rule, |ctx| ctx)
  }

  pub fn define_with<'a, 'buffer: 'a, F>(
    &mut self,
    nt: NTKind,
    rule: impl Into<Vec<ParserBuilderGrammar<TKind, NTKind>>>,
    f: F,
  ) -> GrammarRuleId
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
    let rule = rule.into();
    let expect = rule
      .iter()
      .enumerate()
      .filter_map(|(i, g)| if g.expected { Some(i) } else { None })
      .collect();
    let rule = rule
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
        another_rule: r.another_rule,
        accepter: r.accepter,
        condition: r
          .condition
          .into_resolved_conflict_condition(&mut self.grammars),
      })
      .collect();

    // the new grammar rule
    let gr = self
      .gr_repo
      .get_or_add(self.grammars.get_or_create_nt(nt).clone(), rule);

    // ensure we don't define the same grammar rule twice
    if !self.defined_grs.insert(gr.id().clone()) {
      panic!("Grammar rule already defined: {:?}", gr.id());
    }

    // since this is a define action, overwrite properties
    gr.expect = expect;
    gr.rejecter = ctx.rejecter;
    gr.resolved_conflicts = resolved_conflicts;
    gr.callback = ctx.callback;

    gr.id().clone()
  }

  // TODO: make gr a ref? so that we don't need to clone it outside
  pub fn append(
    self,
    nt: NTKind,
    rule: impl Into<Vec<ParserBuilderGrammar<TKind, NTKind>>>,
  ) -> Self {
    self.append_with(nt, rule, |ctx| ctx)
  }

  pub fn append_with<'a, 'buffer: 'a, F>(
    mut self,
    nt: NTKind,
    rule: impl Into<Vec<ParserBuilderGrammar<TKind, NTKind>>>,
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
    self.define_with(nt, rule, f);
    self
  }

  pub fn resolve_rs<F>(
    mut self,
    reducer_rule_id: GrammarRuleId,
    another_rule_id: GrammarRuleId,
    f: F,
  ) -> Self
  where
    F: FnOnce(
      ReduceShiftResolverOptions<
        TKind,
        NTKind,
        ASTData,
        ErrorType,
        Global,
        LexerActionState,
        LexerErrorType,
      >,
    ) -> ReduceShiftResolverOptions<
      TKind,
      NTKind,
      ASTData,
      ErrorType,
      Global,
      LexerActionState,
      LexerErrorType,
    >,
  {
    let ctx = f(ReduceShiftResolverOptions::default());

    self
      .gr_repo
      .get_mut(&reducer_rule_id)
      .resolved_conflicts
      .push(ResolvedConflict {
        kind: ConflictKind::ReduceShift,
        another_rule: another_rule_id,
        accepter: ctx.accepter,
        condition: ctx
          .condition
          .into_resolved_conflict_condition(&mut self.grammars),
      });

    self
  }

  pub fn resolve_rr<F>(
    mut self,
    reducer_rule_id: GrammarRuleId,
    another_rule_id: GrammarRuleId,
    f: F,
  ) -> Self
  where
    F: FnOnce(
      ReduceReduceResolverOptions<
        TKind,
        NTKind,
        ASTData,
        ErrorType,
        Global,
        LexerActionState,
        LexerErrorType,
      >,
    ) -> ReduceReduceResolverOptions<
      TKind,
      NTKind,
      ASTData,
      ErrorType,
      Global,
      LexerActionState,
      LexerErrorType,
    >,
  {
    let ctx = f(ReduceReduceResolverOptions::default());

    // get reducer rule from gr_repo
    self
      .gr_repo
      .get_mut(&reducer_rule_id)
      .resolved_conflicts
      .push(ResolvedConflict {
        kind: ConflictKind::ReduceReduce,
        another_rule: another_rule_id,
        accepter: ctx.accepter,
        condition: ctx
          .condition
          .into_resolved_conflict_condition(&mut self.grammars),
      });

    self
  }

  /// Set the lexer panic handler. By default the handler is [`default_lexer_panic_handler`].
  pub fn on_lexer_panic<F>(mut self, f: F) -> Self
  where
    F: Fn(&mut TrimmedLexer<TKind, LexerActionState, LexerErrorType>) + 'static,
  {
    self.lexer_panic_handler = Box::new(f);
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
      self.lexer_panic_handler,
      Rc::new(RefCell::new(self.global)),
    )
  }
}
