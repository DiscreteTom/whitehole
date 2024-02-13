use whitehole::lexer::{Action, LexerBuilder};
use whitehole_macros::TokenKind;
use MyKind::*; // use the enum variants directly

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
  let mut lexer = LexerBuilder::<MyKind, MyState>::default()
    .define_with(True, |a| {
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
    .define(False, Action::regex(r"^false").unwrap())
    .build("false");
  // the lexed token should be `False`
  assert!(matches!(lexer.lex().token.unwrap().kind, False));
  // but the action for `True` is evaluated
  assert!(lexer.action_state().evaluated);

  // now with head matcher
  let mut lexer = LexerBuilder::<MyKind, MyState>::default()
    .define_with(True, |a| {
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
      False,
      Action::regex(r"^false")
        .unwrap()
        // only evaluate this action if the first character is `f`
        .head_in(['f']),
    )
    .build("false");
  // the lexed token should be `False`
  assert!(matches!(lexer.lex().token.unwrap().kind, False));
  // and the action for `True` is NOT evaluated
  assert!(!lexer.action_state().evaluated);

  // if an action has no head matcher
  // the action will always be evaluated
  let mut lexer = LexerBuilder::<MyKind, MyState>::default()
    .define_with(True, |a| {
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
      False,
      Action::regex(r"^false")
        .unwrap()
        // only evaluate this action if the first character is `f`
        .head_in(['f']),
    )
    .build("false");
  // the lexed token should be `False`
  assert!(matches!(lexer.lex().token.unwrap().kind, False));
  // but the action for `True` is evaluated
  assert!(lexer.action_state().evaluated);

  // we can use head_not to exclude some characters
  let mut lexer = LexerBuilder::<MyKind, MyState>::default()
    .define_with(Others, |a| {
      a.simple(|input| {
        input.state_mut().evaluated = true;

        if [',', ':', '{', '}', '[', ']'].contains(&(input.rest().chars().next().unwrap())) {
          1
        } else {
          0
        }
      })
      // instead of using `head_in([',', ':', '{', '}', '[', ']'])`
      .head_not(['t', 'f'])
    })
    .define(False, Action::regex(r"^false").unwrap().head_in(['f']))
    .build("false");
  // the lexed token should be `False`
  assert!(matches!(lexer.lex().token.unwrap().kind, False));
  // and the action for `True` is NOT evaluated
  assert!(!lexer.action_state().evaluated);

  // we can also use head_unknown to match any unknown characters
  let mut lexer = LexerBuilder::<MyKind, MyState>::default()
    .define_with(Others, |a| {
      a.simple(|input| {
        input.state_mut().evaluated = true;

        if [',', ':', '{', '}', '[', ']'].contains(&(input.rest().chars().next().unwrap())) {
          1
        } else {
          0
        }
      })
      .head_unknown()
    })
    .define(False, Action::regex(r"^false").unwrap().head_in(['f']))
    .build("false");
  // the lexed token should be `False`
  assert!(matches!(lexer.lex().token.unwrap().kind, False));
  // and the action for `True` is NOT evaluated
  assert!(!lexer.action_state().evaluated);

  // head matcher will take effect in lexing, expectational lexing,
  // peeking and trimming
}
