use std::collections::HashSet;
use whitehole::{
  kind::kind,
  lexer::{
    action::{exact, regex, whitespaces, HeadMatcher},
    builder::LexerBuilder,
  },
};

// define token kinds, make sure it is decorated by `#[kind]`
#[kind]
#[derive(Clone, Default)]
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
fn lex_with_head() {
  // by default the lexer will evaluate all your actions one by one
  // but in some cases we can know which action to use
  // by looking at the first character of the rest of the text.
  // e.g. when lexing a json string, we know that
  // if the first character is `"` then we can use the `string` action
  // if the first character is `t` then we can use the `true` action
  // if the first character is `f` then we can use the `false` action
  // etc.
  // in this case, we can use head matcher to narrow down the actions
  // to speed up the lexing process.

  // let's see if there is no head matcher
  let mut lexer = LexerBuilder::stateful::<MyState>()
    .define_with(
      True,
      regex(r"^true"),
      // mutate the state when the action is evaluated
      // no matter if it's accepted or rejected
      |a| a.prepare(|input| input.state.evaluated = true),
    )
    .define(False, regex(r"^false"))
    .build("false");
  // the lexed token should be `False`
  assert!(matches!(
    lexer.lex().token.unwrap().binding.kind(),
    MyKind::False
  ));
  // but the action for `True` is evaluated
  assert!(lexer.state.evaluated);

  // now with head matcher
  let mut lexer = LexerBuilder::stateful::<MyState>()
    .define_with(True, regex(r"^true"), |a| {
      // mutate the state when the action is evaluated
      // no matter if it's accepted or rejected
      a.prepare(|input| input.state.evaluated = true)
        // only evaluate this action if the first character is `t`
        .unchecked_head_in(['t'])
    })
    .define_with(
      False,
      regex(r"^false"),
      // only evaluate this action if the first character is `f`
      |a| a.unchecked_head_in(['f']),
    )
    .build("false");
  // the lexed token should be `False`
  assert!(matches!(
    lexer.lex().token.unwrap().binding.kind(),
    MyKind::False
  ));
  // and the action for `True` is NOT evaluated
  assert!(!lexer.state.evaluated);

  // if an action has no head matcher
  // the action will always be evaluated
  let mut lexer = LexerBuilder::stateful::<MyState>()
    .define_with(
      True,
      // no head matcher for this action
      regex(r"^true"),
      // mutate the state when the action is evaluated
      // no matter if it's accepted or rejected
      |a| a.prepare(|input| input.state.evaluated = true),
    )
    .define_with(False, regex(r"^false"), |a| {
      // only evaluate this action if the first character is `f`
      a.unchecked_head_in(['f'])
    })
    .build("false");
  // the lexed token should be `False`
  assert!(matches!(
    lexer.lex().token.unwrap().binding.kind(),
    MyKind::False
  ));
  // but the action for `True` is evaluated
  assert!(lexer.state.evaluated);

  // you can use `head_not` to exclude some characters
  let mut lexer = LexerBuilder::stateful::<MyState>()
    .define_with(Others, regex(r"^[^tf]"), |a| {
      a.prepare(|input| input.state.evaluated = true)
        .unchecked_head_not(['t', 'f'])
    })
    .define_with(False, regex(r"^false"), |a| a.unchecked_head_in(['f']))
    .build("false");
  // the lexed token should be `False`
  assert!(matches!(
    lexer.lex().token.unwrap().binding.kind(),
    MyKind::False
  ));
  // and the action for `True` is NOT evaluated
  assert!(!lexer.state.evaluated);

  // you can also use head_unknown to match any unknown characters
  let mut lexer = LexerBuilder::stateful::<MyState>()
    .define_with(Others, regex(r"^[^tf]"), |a| {
      a.prepare(|input| input.state.evaluated = true)
        .unchecked_head_unknown()
    })
    .define_with(False, regex(r"^false"), |a| a.unchecked_head_in(['f']))
    .build("false");
  // the lexed token should be `False`
  assert!(matches!(
    lexer.lex().token.unwrap().binding.kind(),
    MyKind::False
  ));
  // and the action for `True` is NOT evaluated
  assert!(!lexer.state.evaluated);

  // head matcher will take effect in lexing, expectational lexing,
  // peeking and trimming
}

#[test]
fn head_set_by_action_utils() {
  // many action utils already have the head matcher set
  assert!(
    matches!(exact::<(), ()>("true").head(), Some(HeadMatcher::OneOf(set)) if set == &HashSet::from(['t']))
  );
  assert!(
    matches!(whitespaces::<(), ()>().head(), Some(HeadMatcher::OneOf(set)) if set.len() == 25)
  );
}

#[test]
fn utf8_head() {
  // head matcher will work with utf8
  let mut lexer = LexerBuilder::stateful::<MyState>()
    .define_with(True, exact("真"), |a| {
      a.prepare(|input| input.state.evaluated = true)
    })
    .define(False, exact("假"))
    .build("假");
  // the lexed token should be `False`
  assert!(matches!(
    lexer.lex().token.unwrap().binding.kind(),
    MyKind::False
  ));
  // and the action for `True` is NOT evaluated
  assert!(!lexer.state.evaluated);
}
