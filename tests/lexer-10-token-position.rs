use whitehole::lexer::{
  action::regex,
  position::{Position, PositionTransformer},
  token::{token_kind, Range},
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
fn token_position() {
  // for a better performance, the position of each token is not tracked by default.
  // to get the position of a token (instead of an index)
  // you can use PositionTransformer
  let text = "123\n123\n";
  // when creating a PositionTransformer it will calculate the range in byte of each line
  let pt = PositionTransformer::new(text);

  let lexer = LexerBuilder::new()
    .ignore_default(regex(r"^\n"))
    .define(A, regex(r"^123"))
    .build(text);

  let (output, _) = lexer.peek();
  let token = output.token.unwrap();

  // use `transform` to get the position from the index,
  // PositionTransformer will use binary search to find the line index
  // and calculate the character index by counting the characters in the line
  let position = pt.transform(token.range.start, text).unwrap();
  assert!(matches!(
    position,
    Position {
      line: 0,
      character: 0
    }
  ));

  // however, if you want to batch transform the positions of all tokens,
  // calling `transform` for each token is not efficient
  // you can get the line ranges and calculate them by yourself
  let line_ranges = pt.line_ranges();
  assert_eq!(line_ranges.len(), 3);
  assert!(matches!(line_ranges[0], Range { start: 0, end: 4 }));
  assert!(matches!(line_ranges[1], Range { start: 4, end: 8 }));
  assert!(matches!(line_ranges[2], Range { start: 8, end: 8 }));

  // you can also do this in parallel or using multiple threads to improve the performance
}
