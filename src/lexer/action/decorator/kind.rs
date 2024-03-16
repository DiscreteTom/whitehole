use super::AcceptedActionDecoratorContext;
use crate::lexer::{
  action::{ActionInput, ActionOutput, EnhancedActionOutput},
  token::{TokenKind, TokenKindIdBinding},
  Action,
};

impl<Kind, ActionState, ErrorType> Action<Kind, ActionState, ErrorType> {
  /// Set the kind and the data binding for this action.
  /// Use this if your action can only yield one kind.
  /// # Examples
  /// ```
  /// use whitehole::lexer::action::{Action, simple};
  /// use whitehole::lexer::token::TokenKindIdBinding;
  /// use whitehole_macros::TokenKind;
  ///
  /// #[derive(TokenKind, Clone, Debug)]
  /// enum MyKind { A }
  ///
  /// let action: Action<TokenKindIdBinding<MyKind>> = simple(|_| 1).bind(A);
  /// ```
  pub fn bind<NewKind, ViaKind>(
    self,
    kind: ViaKind,
  ) -> Action<TokenKindIdBinding<NewKind>, ActionState, ErrorType>
  where
    ViaKind: TokenKind<TokenKindIdBinding<NewKind>> + Into<TokenKindIdBinding<NewKind>>,
    NewKind: Clone + 'static,
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
  {
    let exec = self.exec;
    let kind = kind.into();
    Action {
      possible_kinds: ViaKind::possible_kinds(),
      head_matcher: self.head_matcher,
      maybe_muted: self.maybe_muted,
      may_mutate_state: self.may_mutate_state,
      exec: Box::new(move |input| {
        exec(input).map(|output| ActionOutput {
          kind: kind.clone(),
          digested: output.digested,
          muted: output.muted,
          error: output.error,
        })
      }),
    }
  }

  /// Set the kind and the data binding for this action by the selector.
  /// Use this if you need to calculate the kind based on the [`ActionInput`] and [`ActionOutput`].
  /// # Examples
  /// ```
  /// use whitehole::lexer::action::{Action, regex};
  /// use whitehole::lexer::token::TokenKindIdBinding;
  /// use whitehole_macros::TokenKind;
  ///
  /// #[derive(TokenKind, Clone, Debug)]
  /// enum MyKind { Odd, Even }
  ///
  /// let action: Action<TokenKindIdBinding<MyKind>> = regex(r"^\d+")
  ///   .unwrap()
  ///   .select(|ctx| {
  ///     if ctx.output.digested % 2 == 0 {
  ///       MyKind::Even
  ///     } else {
  ///       MyKind::Odd
  ///     }
  ///   });
  pub fn select<NewKind, ViaKind, F>(
    self,
    selector: F,
  ) -> Action<TokenKindIdBinding<NewKind>, ActionState, ErrorType>
  where
    ViaKind: Into<TokenKindIdBinding<NewKind>> + TokenKind<TokenKindIdBinding<NewKind>>,
    Kind: 'static,
    ActionState: 'static,
    ErrorType: 'static,
    F: Fn(
        AcceptedActionDecoratorContext<
          // user can't mutate the input
          &ActionInput<ActionState>,
          // output is consumed except the error
          EnhancedActionOutput<Kind, &Option<ErrorType>>,
        >,
      ) -> ViaKind
      + 'static,
  {
    let exec = self.exec;
    Action {
      possible_kinds: ViaKind::possible_kinds(),
      head_matcher: self.head_matcher,
      maybe_muted: self.maybe_muted,
      may_mutate_state: self.may_mutate_state,
      exec: Box::new(move |input| {
        exec(input).map(|output| {
          ActionOutput {
            kind: selector(AcceptedActionDecoratorContext {
              output: EnhancedActionOutput::new(
                input,
                // construct a new ActionOutput
                ActionOutput {
                  // consume the original output.kind
                  kind: output.kind,
                  digested: output.digested,
                  muted: output.muted,
                  // but don't consume the error
                  error: &output.error,
                },
              ),
              input,
            })
            .into(),
            digested: output.digested,
            muted: output.muted,
            error: output.error,
          }
        })
      }),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lexer::action::{regex, simple};
  use whitehole_macros::_TokenKind;
  use Num::*;

  #[derive(_TokenKind, Clone)]
  enum Num {
    Even,
    Odd,
  }

  fn test_runner<F>(f: F)
  where
    F: Fn(Action<()>) -> Action<Num>,
  {
    let action = f(regex(r"^\d+").unwrap().head_unknown().mute(true));

    // ensure the possible kinds is set
    assert_eq!(action.possible_kinds.len(), 2);
    assert!(action.possible_kinds.contains(&Even.id()));
    assert!(action.possible_kinds.contains(&Odd.id()));

    // ensure other fields are not changed
    assert!(action.maybe_muted);
    assert!(matches!(
      action.head_matcher,
      Some(ActionInputRestHeadMatcher::Unknown)
    ));

    // ensure the result is correct
    assert!(matches!(
      action.exec(&mut ActionInput::new("123", 0, &mut ())),
      Some(ActionOutput {
        kind: Odd,
        digested: 3,
        muted: true,
        error: None
      })
    ));
    assert!(matches!(
      action.exec(&mut ActionInput::new("124", 0, &mut ())),
      Some(ActionOutput {
        kind: Even,
        digested: 3,
        muted: true,
        error: None
      })
    ));
  }

  #[test]
  fn action_kind_ids() {
    test_runner(|a| {
      a.kind_ids([Even.id(), Odd.id()]).select(|ctx| {
        if ctx.output.content().parse::<u32>().unwrap() % 2 == 0 {
          Even
        } else {
          Odd
        }
      })
    });
  }

  #[test]
  fn action_kinds() {
    test_runner(|a| {
      a.kinds([Even, Odd]).select(|ctx| {
        if ctx.output.content().parse::<u32>().unwrap() % 2 == 0 {
          Even
        } else {
          Odd
        }
      })
    });
  }

  #[test]
  fn action_bind() {
    let action: Action<Num> = simple(|_| 1).bind(Even);
    assert_eq!(action.possible_kinds.len(), 1);
    assert!(action.possible_kinds.contains(&Even.id()));
    assert!(matches!(
      action.exec(&mut ActionInput::new("A", 0, &mut ())),
      Some(ActionOutput {
        kind: Even,
        digested: 1,
        muted: false,
        error: None
      })
    ));
  }
}
