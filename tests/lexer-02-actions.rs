use whitehole::{
  kind::whitehole_kind,
  lexer::{
    action::{comment, exact_chars, exact_vec, regex, simple, whitespaces, word_vec},
    builder::LexerBuilder,
    token::Range,
  },
};

// define token kinds, make sure it is decorated by `#[whitehole_kind]`
#[whitehole_kind]
#[derive(Clone, Default)]
enum MyKind {
  #[default]
  Anonymous,
  A,
  B,
  C,
  D,
}

#[test]
fn action_orders() {
  let mut lexer = LexerBuilder::new()
    // first defined actions have higher priority
    .define(A, regex(r"^.")) // highest priority
    .define(B, regex(r"^."))
    .define(C, regex(r"^.")) // lowest priority
    .build("aa");

  // lexing will always emit `A`
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.binding.kind(), MyKind::A));
  let token = lexer.lex().token.unwrap();
  assert!(matches!(token.binding.kind(), MyKind::A));
}

// define state which can be shared between actions
#[derive(Default, Clone)]
struct MyState {
  reject: bool,
}

#[test]
fn action_decorators() {
  let mut lexer = LexerBuilder::stateful::<MyState>()
    // you can use `define_with` to apply a decorator to all the actions in the `define` call
    .define_with(
      Anonymous,
      // to mute an action (just like `LexerBuilder::ignore`), you can use `mute`
      regex(r"^\s+"),
      |a| a.mute(),
    )
    .define_with(
      B,
      // to reject an action after the action is executed and accepted, you can use `reject` or `reject_if`
      regex(r"^b"),
      |a| a.reject_if(|ctx| !ctx.rest().is_empty()),
    )
    .define_with(
      C,
      // to reject an action before the action is executed, you can use `prevent`
      regex(r"^c"),
      |a| a.prevent(|input| input.state.reject),
    )
    .define_with(
      D,
      // use `then` to run a callback if this action is accepted
      // this is usually used to modify lexer's state
      regex(r"^d"),
      |a| {
        a.then(|ctx| {
          ctx.input.state.reject = true;
        })
        // yes you can apply multi decorators to an action
        .prevent(|input| input.state.reject)
      },
    )
    .build("a b c");

  // // the first lex should be accepted but with error set
  // let mut errors = vec![];
  // let res = lexer.lex_with(|o| o.errors().to(&mut errors));
  // let token = res.token.unwrap();
  // assert!(matches!(token.binding.kind(), MyKind::A));
  // assert_eq!(res.digested, 1);
  // assert!(matches!(errors[0], ("error", Range { start: 0, end: 1 })));

  // // the second lex should be rejected but still digest some characters
  // errors.clear();
  // let res = lexer.lex_with(|o| o.errors().to(&mut errors));
  // assert!(matches!(res.token, None));
  // assert_eq!(res.digested, 1); // digest one whitespace
  // assert_eq!(errors.len(), 0); // no new error

  // // create a new lexer with the same actions and a new input
  // let mut lexer = lexer.reload("c d c");

  // // the first lex should be accepted
  // let token = lexer.lex().token.unwrap();
  // assert!(matches!(token.binding.kind(), MyKind::C));
  // assert_eq!(token.range.start, 0);
  // assert_eq!(token.range.end, 1);

  // // the second lex should be accepted and will change the state
  // let token = lexer.lex().token.unwrap();
  // assert!(matches!(token.binding.kind(), MyKind::D));
  // assert_eq!(token.range.start, 2);
  // assert_eq!(token.range.end, 3);
  // assert_eq!(lexer.state.reject, true);

  // // the third lex should be rejected
  // let res = lexer.lex();
  // assert!(matches!(res.token, None));
}

#[test]
fn action_utils() {
  // as a best practice, you should use action utils as much as possible,
  // especially for tokens with fixed content (like keywords, operators, etc.),
  // and use `simple` or `regex` for dynamic tokens (like numbers, identifiers, strings, etc.).
  // here are the most common action utils:
  LexerBuilder::new()
    // usually whitespaces and comments won't emit any token, so you can use `ignore_default`
    .ignore_default(whitespaces())
    .ignore_default([comment("//", "\n"), comment("/*", "*/")])
    // for keywords and operators, they are literal and don't need a kind,
    // so you can use `append_default` to bind them with the default token kind.
    // for keywords, there should be a word boundary after the keyword,
    // you can use `word` and `word_vec` to ensure the word boundary.
    .append_default(word_vec!["true", "false"])
    // for multi-char operators, you can use `exact_vec` to match them exactly, no lookahead needed.
    .append_default(exact_vec!["++", "--"])
    // for single-char operators, you can use `exact_chars` to match them exactly, no lookahead needed.
    .append_default(exact_chars("+-*/()?:;"))
    // for dynamic tokens, you can use `simple` or `regex` to write your own action,
    .define(A, regex(r"^\s+"))
    .define(A, simple(|input| input.rest().len()))
    .build("a b c");
}
