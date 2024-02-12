use whitehole::lexer::{Action, Builder};
use whitehole_macros::TokenKind;

// define token kinds
// make sure it implements `TokenKind` and `Clone`.
#[derive(TokenKind, Clone, Default)]
enum MyKind {
  #[default]
  Anonymous,
  True,
  False,
  Others,
}

#[derive(Default, Clone)]
struct MyState {
  evaluated: bool,
}

#[test]
fn lex_with_head_matcher() {
  // by default the lexer will evaluate our actions one by one
  // but in some cases we can know which action to use
  // by looking at the first character of the rest of the buffer.
  // e.g. when lexing a json string, we know that
  // if the first character is `"` then we can use the string action
  // if the first character is `t` then we can use the true action
  // if the first character is `f` then we can use the false action
  // etc.
  // in this case, we can use head matcher to narrow down the actions

  // let's see if there is no head matcher
  let mut lexer = Builder::<MyKind, MyState, ()>::default()
    .define_from(MyKind::True, |a| {
      a.simple(|input| {
        // mutate the action state when the action is evaluated
        // no matter if it's accepted or rejected
        input.state_mut().evaluated = true;

        let pattern = "true";
        if input.rest().starts_with(pattern) {
          pattern.len()
        } else {
          0
        }
      })
    })
    .define(MyKind::False, Action::regex(r"^false").unwrap())
    .build("false");
  // the lexed token should be `MyKind::False`
  assert!(matches!(lexer.lex().token.unwrap().kind, MyKind::False));
  // but the action for `MyKind::True` is evaluated
  assert!(lexer.action_state().evaluated);

  // now with head matcher
  let mut lexer = Builder::<MyKind, MyState, ()>::default()
    .define_from(MyKind::True, |a| {
      a.simple(|input| {
        // mutate the action state when the action is evaluated
        // no matter if it's accepted or rejected
        input.state_mut().evaluated = true;

        let pattern = "true";
        if input.rest().starts_with(pattern) {
          pattern.len()
        } else {
          0
        }
      })
      // only evaluate this action if the first character is `t`
      .head_in(['t'])
    })
    .define(
      MyKind::False,
      Action::regex(r"^false")
        .unwrap()
        // only evaluate this action if the first character is `f`
        .head_in(['f']),
    )
    .build("false");
  // the lexed token should be `MyKind::False`
  assert!(matches!(lexer.lex().token.unwrap().kind, MyKind::False));
  // and the action for `MyKind::True` is NOT evaluated
  assert!(!lexer.action_state().evaluated);

  // if an action has no head matcher
  // the action will always be evaluated
  let mut lexer = Builder::<MyKind, MyState, ()>::default()
    .define_from(MyKind::True, |a| {
      a.simple(|input| {
        // mutate the action state when the action is evaluated
        // no matter if it's accepted or rejected
        input.state_mut().evaluated = true;

        let pattern = "true";
        if input.rest().starts_with(pattern) {
          pattern.len()
        } else {
          0
        }
      })
      // no head matcher for this action
    })
    .define(
      MyKind::False,
      Action::regex(r"^false")
        .unwrap()
        // only evaluate this action if the first character is `f`
        .head_in(['f']),
    )
    .build("false");
  // the lexed token should be `MyKind::False`
  assert!(matches!(lexer.lex().token.unwrap().kind, MyKind::False));
  // but the action for `MyKind::True` is evaluated
  assert!(lexer.action_state().evaluated);

  // we can use head_not to exclude some characters
  let mut lexer = Builder::<MyKind, MyState, ()>::default()
    .define_from(MyKind::Others, |a| {
      a.simple(|input| {
        input.state_mut().evaluated = true;

        if [',', ':', '{', '}', '[', ']'].contains(&(input.rest().as_bytes()[0] as char)) {
          1
        } else {
          0
        }
      })
      // instead of using `head_in([',', ':', '{', '}', '[', ']'])`
      .head_not(['t', 'f'])
    })
    .define(
      MyKind::False,
      Action::regex(r"^false").unwrap().head_in(['f']),
    )
    .build("false");
  // the lexed token should be `MyKind::False`
  assert!(matches!(lexer.lex().token.unwrap().kind, MyKind::False));
  // and the action for `MyKind::True` is NOT evaluated
  assert!(!lexer.action_state().evaluated);

  // we can also use head_unknown to match any unknown characters
  let mut lexer = Builder::<MyKind, MyState, ()>::default()
    .define_from(MyKind::Others, |a| {
      a.simple(|input| {
        input.state_mut().evaluated = true;

        if [',', ':', '{', '}', '[', ']'].contains(&(input.rest().as_bytes()[0] as char)) {
          1
        } else {
          0
        }
      })
      .head_unknown()
    })
    .define(
      MyKind::False,
      Action::regex(r"^false").unwrap().head_in(['f']),
    )
    .build("false");
  // the lexed token should be `MyKind::False`
  assert!(matches!(lexer.lex().token.unwrap().kind, MyKind::False));
  // and the action for `MyKind::True` is NOT evaluated
  assert!(!lexer.action_state().evaluated);

  // head matcher will take effect in lexing, expectational lexing,
  // peeking and trimming
}
