use whitehole::lexer::{
  action::{chars, exact, exact_sub, regex, simple, sub, SubAction},
  token::token_kind,
  LexerBuilder,
};

// define token kinds, make sure it is decorated by `#[token_kind]`
#[token_kind]
#[derive(Clone, Default)]
enum MyKind {
  #[default]
  Anonymous,
  A,
}

#[test]
fn sub_action() {
  // SubAction is a simpler version of Action,
  // you can't mutate the action state in a SubAction,
  // and you only need to return how many bytes you want to consume.
  LexerBuilder::new()
    .define(
      A,
      // you can provide an array of SubActions here
      [
        // you can use `SubAction::new` to create a new SubAction
        SubAction::new(|input| Some(input.rest().len())),
        // or use `sub` for short
        sub(|input| Some(input.rest().len())),
        // some action utils will return SubAction instead of Action
        simple(|input| input.rest().len()),
        chars(|ch| ch.is_whitespace()),
        exact_sub("sub"),
        regex("regex"),
      ],
    )
    // SubActions can be converted into Actions via `into` or `into_action`
    .define(
      A,
      [
        // if you just want to convert a SubAction into an Action, just use `into`
        regex("regex").into(),
        // if you want to apply some decorators to the Action, use `into_action` for better type inference
        regex("regex").into_action().mute(),
      ],
    )
    // multiple SubActions can be combined into a complex SubAction
    .define(
      A,
      [
        // use `|` to combine SubActions, execute the next SubAction if the previous one is rejected
        chars(|ch| ch == &'a') | chars(|ch| ch == &'b'),
        // use `+` to combine SubActions, execute the next SubAction if the previous one is accepted
        chars(|ch| ch == &'c') + chars(|ch| ch == &'d'),
      ],
    )
    // you can also use `+` to combine an Action and a SubAction, which will generate an Action,
    // the generated Action will inherit properties of the original Action (especially the head matcher)
    .define(
      A,
      [
        // hex integer literal
        exact("0x") + chars(|ch| ch.is_ascii_hexdigit()),
        // simple double quoted string literal
        exact("\"")
          + (chars(|ch| ch != &'\\' && ch != &'\"') | regex(r"\r|\n\t|"))
          + exact_sub("\""),
      ],
    );
}
