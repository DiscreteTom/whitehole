use whitehole::{
  combinator::{eat, next},
  parser::Parser,
};

fn main() {
  let double_hex = || {
    // Repeat a combinator with `*`.
    (next(|c| c.is_ascii_hexdigit()) * 2)
      // Convert the matched content to `u8`.
      .select(|ctx| u8::from_str_radix(ctx.content(), 16).unwrap())
      // Wrap `u8` to `(u8,)`, this is required by `+` below.
      .tuple()
  };

  // Concat multiple combinators with `+`.
  // Tuple values will be concatenated into a single tuple.
  // Here `() + (u8,) + (u8,) + (u8,)` will be `(u8, u8, u8)`.
  let entry = eat('#') + double_hex() + double_hex() + double_hex();

  let mut parser = Parser::builder().entry(entry).build("#FFA500");
  let output = parser.next().unwrap();
  assert_eq!(output.digested, 7);
  assert_eq!(output.value, (0xFF, 0xA5, 0x00));
}
