use whitehole::lexer::{
  action::regex,
  position::{Position, PositionTransformer},
  token::Range,
  LexerBuilder,
};
use whitehole_macros::TokenKind;
use MyKind::*; // use the enum variants directly

// define token kinds
// make sure it implements `TokenKind` and `Clone`.
#[derive(TokenKind, Clone)]
enum MyKind {
  Anonymous,
  A,
}

#[test]
fn token_position() {
  // for a better performance, lexer doesn't keep track of the position of each token.
  // token is ephemeral, it has information about the index of the buffer when it is created.

  // to get the position of a token (instead of an index)
  // we can use PositionTransformer
  let text = "123\n123\n";
  // the position transformer will calculate the index of each line
  // so we can use it to get the position of a token
  let pt = PositionTransformer::new(text);

  let lexer = LexerBuilder::<MyKind>::default()
    .ignore(regex(r"^\n").unwrap().bind(Anonymous))
    .define(A, regex(r"^123").unwrap())
    .build(text);

  let (output, _) = lexer.peek();
  let token = output.token.unwrap();

  // use `transform` to get the position from the index
  // it will use binary search to find the line index
  let position = pt.transform(token.range.start).unwrap();
  assert!(matches!(position, Position { line: 1, column: 1 }));

  // however, if we want to batch transform the positions of all tokens
  // calling `transform` for each token is not efficient
  // we can get the line ranges and calculate them by ourselves
  let line_ranges = pt.line_ranges();
  assert_eq!(line_ranges.len(), 3);
  assert!(matches!(line_ranges[0], Range { start: 0, end: 4 }));
  assert!(matches!(line_ranges[1], Range { start: 4, end: 8 }));
  assert!(matches!(line_ranges[2], Range { start: 8, end: 8 }));
}
